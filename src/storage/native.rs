use bevy::{
    ecs::world::CommandQueue,
    prelude::*,
    tasks::{block_on, poll_once, IoTaskPool},
};

use crate::signals::SyltSignal;

pub struct SyltStoragePlugin;

impl Plugin for SyltStoragePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, ensure_directories);
        app.add_systems(
            Update,
            (
                write_data_system,
                handle_write_data_task,
                read_data_task,
                handle_read_data_task,
            ),
        );
    }
}

fn ensure_directories() {
    let project_qualifier = std::env::var("PROJECT_QUALIFIER")
        .unwrap_or_else(|_| "sylt".to_string());
    let project_organization = std::env::var("PROJECT_ORGANIZATION")
        .unwrap_or_else(|_| "sylt".to_string());
    let project_name =
        std::env::var("PROJECT_NAME").unwrap_or_else(|_| "sylt".to_string());

    let project_dir = directories::ProjectDirs::from(
        project_qualifier.to_lowercase().as_str(),
        project_organization.to_lowercase().as_str(),
        project_name.to_lowercase().as_str(),
    )
    .unwrap();

    let _ = std::fs::create_dir_all(project_dir.config_dir());
}

#[derive(Component)]
struct SyltStorageWriteTask(pub bevy::tasks::Task<CommandQueue>);

fn handle_write_data_task(
    mut cmd: Commands,
    mut sylt_storage_write_tasks: Query<(Entity, &mut SyltStorageWriteTask)>,
) {
    for (entity, mut task) in sylt_storage_write_tasks.iter_mut() {
        if let Some(mut command_queue) = block_on(poll_once(&mut task.0)) {
            cmd.append(&mut command_queue);
            cmd.entity(entity).despawn();
        }
    }
}

fn write_data_system(
    mut cmd: Commands,
    mut sylt_signal_reader: EventReader<SyltSignal>,
) {
    for event in sylt_signal_reader.read() {
        if let SyltSignal::SaveFile { key, data } = event {
            let key = key.clone();
            let value = data.clone();

            let project_qualifier = std::env::var("PROJECT_QUALIFIER")
                .unwrap_or_else(|_| "sylt".to_string());
            let project_organization = std::env::var("PROJECT_ORGANIZATION")
                .unwrap_or_else(|_| "sylt".to_string());
            let project_name = std::env::var("PROJECT_NAME")
                .unwrap_or_else(|_| "sylt".to_string());

            let project_dir = directories::ProjectDirs::from(
                project_qualifier.to_lowercase().as_str(),
                project_organization.to_lowercase().as_str(),
                project_name.to_lowercase().as_str(),
            )
            .unwrap();

            let io_task_pool = IoTaskPool::get();

            let task = io_task_pool.spawn(async move {
                let mut command_queue = CommandQueue::default();

                let file_path = project_dir.config_dir().join(key.to_string());

                match std::fs::write(&file_path, value.as_bytes()) {
                    Ok(_) => {
                        debug!("File saved: {}", file_path.display());
                        command_queue.push(move |world: &mut World| {
                            world.send_event::<SyltSignal>(
                                SyltSignal::FileSaved { key },
                            );
                        });
                    }
                    Err(e) => {
                        debug!("Failed to write file {}: {}", key, e);
                        command_queue.push(move |world: &mut World| {
                            world.send_event::<SyltSignal>(
                                SyltSignal::SaveFileError {
                                    key,
                                    message: e.to_string().into(),
                                },
                            );
                        });
                    }
                }

                command_queue
            });

            cmd.spawn(SyltStorageWriteTask(task));
        }
    }
}

#[derive(Component)]
struct SyltStorageReadTask(pub bevy::tasks::Task<CommandQueue>);

fn handle_read_data_task(
    mut cmd: Commands,
    mut settings_write_tasks: Query<(Entity, &mut SyltStorageReadTask)>,
) {
    for (entity, mut task) in settings_write_tasks.iter_mut() {
        if let Some(mut command_queue) = block_on(poll_once(&mut task.0)) {
            cmd.append(&mut command_queue);
            cmd.entity(entity).despawn();
        }
    }
}

fn read_data_task(
    mut cmd: Commands,
    mut sylt_signal_reader: EventReader<SyltSignal>,
) {
    for event in sylt_signal_reader.read() {
        let event = event.clone();
        let project_qualifier = std::env::var("PROJECT_QUALIFIER")
            .unwrap_or_else(|_| "sylt".to_string());
        let project_organization = std::env::var("PROJECT_ORGANIZATION")
            .unwrap_or_else(|_| "sylt".to_string());
        let project_name = std::env::var("PROJECT_NAME")
            .unwrap_or_else(|_| "sylt".to_string());

        let io_task_pool = IoTaskPool::get();

        if let SyltSignal::LoadFile { key } = event {
            let project_dir = directories::ProjectDirs::from(
                project_qualifier.to_lowercase().as_str(),
                project_organization.to_lowercase().as_str(),
                project_name.to_lowercase().as_str(),
            )
            .unwrap();

            let task = io_task_pool.spawn(async move {
                let mut command_queue = CommandQueue::default();
                let file_path = project_dir.config_dir().join(key.to_string());
                match std::fs::read_to_string(&file_path) {
                    Ok(data) => {
                        debug!("File loaded: {}", file_path.display());
                        command_queue.push(move |world: &mut World| {
                            world.send_event::<SyltSignal>(
                                SyltSignal::FileLoaded {
                                    key,
                                    data: data.into(),
                                },
                            );
                        });
                    }
                    Err(e) => {
                        debug!("Failed to read file {}: {}", key, e);
                        command_queue.push(move |world: &mut World| {
                            world.send_event::<SyltSignal>(
                                SyltSignal::LoadFileError {
                                    key,
                                    message: e.to_string().into(),
                                },
                            );
                        });
                    }
                };
                command_queue
            });

            cmd.spawn(SyltStorageReadTask(task));
        }
    }
}
