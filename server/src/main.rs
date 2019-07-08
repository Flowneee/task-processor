use actix_web::{middleware, web, App, Error, HttpResponse, HttpServer};
use futures::{future::ok, Future, Stream};
use serde_json::json;

mod app;
mod database;
mod plugin;
mod task;

use app::{App as ServerApp, AppConfig};
use task::{TaskInput, TaskOutput, TaskState};

/// Execute task with specified id.
fn execute_task(id: u64, state: web::Data<ServerApp>) {
    // this lock make sure that plugins won't reload
    let _execute_lock = state.stop_tasks_flag.read().unwrap();
    // get task data
    let task_data = {
        let task = state
            .database
            .get(id)
            .unwrap_or_else(|| unreachable!("Task was not set in 'post_task'"));
        let task = match task {
            TaskState::New(x) => x,
            _ => panic!("Task is in incorrect state"),
        };
        let _ = state
            .database
            .insert(id, TaskState::InProgress(task.clone()));
        task
    };
    let plugins_lock = state.plugins.read().unwrap();
    let plugin = match plugins_lock.get(&task_data.module) {
        Some(x) => x,
        None => {
            // log unknown plugin error
            log::error!("Unknown plugin '{}' for task {}", task_data.module, id);
            // store this error into database
            let database = state.database.clone();
            let err = format!("Unknown plugin '{}'", task_data.module);
            let _ = database.insert(
                id,
                TaskState::Done(TaskOutput {
                    input: task_data,
                    result: Err(err),
                }),
            );
            return;
        }
    };
    // unwrap is safe here, because this JSON was retrieved from text
    let task_result = plugin.call(&serde_json::to_string(&task_data.data).unwrap());
    // log result
    log::info!("Task {} finished with result '{:?}", id, task_result);
    // store result
    let _ = state.database.insert(
        id,
        TaskState::Done(TaskOutput {
            input: task_data,
            result: task_result,
        }),
    );
}

/// Post new task.
fn post_task(
    pl: web::Payload,
    state: web::Data<ServerApp>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    pl.concat2().from_err().and_then(move |body| {
        let task = match serde_json::from_slice::<TaskInput>(&body) {
            Ok(x) => x,
            Err(err) => {
                log::error!("Failed to parse task: {}", err);
                return Ok(HttpResponse::BadRequest()
                    .content_type("text/plain")
                    .body("Cannot parse JSON"));
            }
        };
        let new_id = {
            let new_id = state.database.generate_id();
            let _ = state.database.insert(new_id, TaskState::New(task.clone()));
            new_id
        };
        // log incoming task
        log::info!(
            "Recieved task '{}'. Assigned ID: {}",
            serde_json::to_string(&task).unwrap(),
            new_id
        );
        // run task in separate future
        tokio::spawn(ok((new_id, state)).and_then(|(id, state)| {
            execute_task(id, state);
            Ok(())
        }));
        // return id of the task
        let response = json!({ "id": new_id });
        return Ok(HttpResponse::Ok().json(response));
    })
}

/// Get list of registered tasks.
fn get_tasks(state: web::Data<ServerApp>) -> impl Future<Item = HttpResponse, Error = Error> {
    ok(()).and_then(move |()| {
        let data = state.database.dump_to_json();
        Ok(HttpResponse::Ok().json(data))
    })
}

/// Get list of loaded plugins.
fn get_plugins(state: web::Data<ServerApp>) -> impl Future<Item = HttpResponse, Error = Error> {
    ok(()).and_then(move |_| {
        let plugins_names = state
            .plugins
            .read()
            .unwrap()
            .keys()
            .cloned()
            .collect::<Vec<String>>();
        Ok(HttpResponse::Ok().json(plugins_names))
    })
}

/// Refresh plugins.
fn refresh_plugins(state: web::Data<ServerApp>) {
    let plugins_path = state.config.plugins_path.clone();
    // receive execute lock, which stops any tasks from execute
    let _execute_lock = state.stop_tasks_flag.write().unwrap();
    let mut plugins = state.plugins.write().unwrap();
    // log before plugins refreshed
    log::info!(
        "Plugins refresh requested. Path to plugins folder: {}. Current list of plugins: {}",
        plugins_path,
        plugins.keys().cloned().collect::<Vec<String>>().join(", ")
    );
    // reload plugins
    plugins.clear();
    plugins.extend(plugin::load_plugins(plugins_path).unwrap());
    // log after plugins refreshed
    log::info!(
        "Plugins refreshed. Current list of plugins: {}",
        plugins.keys().cloned().collect::<Vec<String>>().join(", ")
    );
}

fn main() -> std::io::Result<()> {
    env_logger::init();

    let config = AppConfig::from_clap();
    log::info!("Application configuration: {:?}", config);
    let addr = config.bind_address.clone();
    let app = ServerApp::from_config(config).unwrap();

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .data(app.clone())
            .service(
                web::resource("/tasks")
                    .route(web::post().to_async(post_task))
                    .route(web::get().to_async(get_tasks)),
            )
            .service(web::resource("/plugins").route(web::get().to_async(get_plugins)))
            .service(web::resource("/plugins/reload").route(web::post().to(refresh_plugins)))
    })
    .bind(addr)?
    .run()
}
