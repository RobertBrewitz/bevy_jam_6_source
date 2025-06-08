use std::sync::Arc;

use bevy::prelude::*;

use indexed_db_futures::prelude::*;
use serde_json::to_string;
use wasm_bindgen::prelude::*;

use shared::prelude::*;

const INDEX_DB_VERSION: u32 = 1;
const STREAM_DB_TABLE: &str = "streams";

pub struct SyltStoragePlugin;

impl Plugin for SyltStoragePlugin {
    fn build(&self, app: &mut App) {}
}

async fn get_index_db() -> Result<IdbDatabase, String> {
    let db_req_ok = IdbDatabase::open_u32("acg", INDEX_DB_VERSION);

    if db_req_ok.is_err() {
        web_sys::console::log_1(&JsValue::from_str("Error opening db"));
        return Err("failed to open db".to_string());
    }

    let mut db_req = db_req_ok.unwrap();

    db_req.set_on_upgrade_needed(Some(
        |evt: &IdbVersionChangeEvent| -> Result<(), JsValue> {
            if let None = evt.db().object_store_names().find(|n| n == "acg") {
                let streams_store =
                    evt.db().create_object_store(STREAM_DB_TABLE)?;
                streams_store.create_index(
                    "streamName",
                    &IdbKeyPath::new(JsValue::from_str("streamName")),
                )?;
            }

            Ok(())
        },
    ));

    let unwrapped = db_req.await;

    if unwrapped.is_err() {
        web_sys::console::log_1(&JsValue::from_str("Error opening db"));
        return Err("Error opening database".to_string());
    }

    Ok(unwrapped.unwrap())
}

async fn get_stream_data(stream: &Stream) -> Result<Vec<Stream>, String> {
    let mut data_stream_ids = vec![];
    let mut streams = vec![];

    for event in stream.events.clone() {
        if event.refs.is_some() {
            let stream_refs = event.refs.unwrap();

            for stream_ref in stream_refs.iter() {
                match stream_ref.action.as_ref() {
                    "add" => {
                        data_stream_ids
                            .append(&mut stream_ref.stream_ids.clone());
                    }
                    "remove" => {
                        data_stream_ids = data_stream_ids
                            .into_iter()
                            .filter(|id| stream_ref.stream_ids.contains(id))
                            .collect();
                    }
                    "replace" => {
                        data_stream_ids = stream_ref.stream_ids.clone();
                    }
                    _ => (),
                }
            }
        }

        data_stream_ids.sort();
        data_stream_ids.dedup();
    }

    if data_stream_ids.len() > 0 {
        let db = get_index_db().await.unwrap();
        let tx: IdbTransaction = db
            .transaction_on_one_with_mode(
                STREAM_DB_TABLE,
                IdbTransactionMode::Readwrite,
            )
            .unwrap();
        let store: IdbObjectStore = tx.object_store(STREAM_DB_TABLE).unwrap();

        for id in data_stream_ids {
            let item = store.get_owned(id.as_ref()).unwrap().await.unwrap();
            let stream: Stream =
                serde_wasm_bindgen::from_value(item.unwrap()).unwrap();
            streams.push(stream);
        }

        let result = tx.await.into_result();

        if result.is_err() {
            web_sys::console::log_1(&JsValue::from_str(&format!(
                "Error writing events: {:?}",
                result.err().unwrap()
            )));
            return Err("Error".to_string());
        }

        return Ok(streams);
    }

    Ok(vec![])
}

pub async fn process_events(events: &Vec<StreamEvent>) -> Result<(), String> {
    let db = get_index_db().await.unwrap();
    let tx: IdbTransaction = db
        .transaction_on_one_with_mode(
            STREAM_DB_TABLE,
            IdbTransactionMode::Readwrite,
        )
        .unwrap();
    let store: IdbObjectStore = tx.object_store(STREAM_DB_TABLE).unwrap();

    for event in events {
        let item = store
            .get_owned(event.stream_id.clone().as_ref())
            .unwrap()
            .await
            .unwrap();

        if item.is_some() {
            let mut stream: Stream =
                serde_wasm_bindgen::from_value(item.unwrap()).unwrap();
            let stream_data_req = get_stream_data(&stream).await;
            if stream_data_req.is_err() {
                return Err(String::from("Failed to request data"));
            }
            let validation = validate_stream(
                Some(&stream),
                Some(&stream_data_req.unwrap()),
                event,
            );
            if validation.is_ok() {
                stream.events.push(event.clone());
                // put stream
                let val_into = serde_wasm_bindgen::to_value(&stream);
                if val_into.is_err() {
                    web_sys::console::log_1(&JsValue::from_str(
                        "Error serializing event",
                    ));
                    return Err("Error serializing event".to_string());
                }
                let store = store.put_key_val(
                    &JsValue::from_str(&event.stream_id),
                    &val_into.unwrap(),
                );
                if store.is_err() {
                    web_sys::console::log_1(&JsValue::from_str(&format!(
                        "Error writing event: {:?}",
                        store.err().unwrap()
                    )));
                    return Err("Error".to_string());
                }
            } else {
                return Err(String::from(validation.err().unwrap()));
            }
        } else {
            let stream_data_req = get_stream_data(&event.into()).await;
            if stream_data_req.is_err() {
                return Err(String::from("Failed to request data"));
            }
            let validation =
                validate_stream(None, Some(&stream_data_req.unwrap()), event);
            if validation.is_ok() {
                let stream: Stream = event.into();
                let val_into = serde_wasm_bindgen::to_value(&stream);
                if val_into.is_err() {
                    web_sys::console::log_1(&JsValue::from_str(
                        "Error serializing event",
                    ));
                    return Err("Error serializing event".to_string());
                }
                let store = store.put_key_val(
                    &JsValue::from_str(&event.stream_id),
                    &val_into.unwrap(),
                );
                if store.is_err() {
                    web_sys::console::log_1(&JsValue::from_str(&format!(
                        "Error writing event: {:?}",
                        store.err().unwrap()
                    )));
                    return Err("Error".to_string());
                }
            } else {
                return Err(String::from(validation.err().unwrap()));
            }
        }
    }

    // close tx
    let result = tx.await.into_result();

    if result.is_err() {
        web_sys::console::log_1(&JsValue::from_str(&format!(
            "Error writing events: {:?}",
            result.err().unwrap()
        )));
        return Err("Error".to_string());
    }

    Ok(())
}

#[wasm_bindgen]
pub async fn command(command_str: &str) -> String {
    let command: CommandArguments = serde_json::from_str(command_str).unwrap();
    let events = create_events(Arc::from("offline"), command);
    let res = process_events(&events).await;
    if res.is_err() {
        return to_string(&CommandOutput {
            events: vec![],
            error: Some(Arc::from("Failed to write events")),
        })
        .unwrap();
    }
    let parse_json = to_string(&CommandOutput {
        events,
        error: None,
    });

    if parse_json.is_err() {
        return to_string(&CommandOutput {
            events: vec![],
            error: Some(Arc::from("Failed to parse json")),
        })
        .unwrap();
    }

    parse_json.unwrap()
}

#[wasm_bindgen]
pub async fn get_data(stream_id: &str) -> String {
    let db = get_index_db().await.unwrap_throw();
    let tx = db.transaction_on_one(STREAM_DB_TABLE).unwrap_throw();
    let store = tx.object_store(STREAM_DB_TABLE).unwrap_throw();
    let item = store
        .get_owned(&serde_wasm_bindgen::to_value(&stream_id).unwrap_throw())
        .unwrap_throw()
        .await
        .unwrap_throw();
    let _ = tx;
    if item.is_some() {
        let stream: Stream =
            serde_wasm_bindgen::from_value(item.unwrap_throw()).unwrap_throw();
        let data = get_stream_data(&stream).await;
        let parse_json = to_string(&data);
        if parse_json.is_ok() {
            return parse_json.unwrap_throw();
        }
    }
    return String::from("[]");
}

#[wasm_bindgen]
pub async fn get_stream(stream_id: &str) -> String {
    let db = get_index_db().await.unwrap_throw();
    let tx = db.transaction_on_one(STREAM_DB_TABLE).unwrap_throw();
    let store = tx.object_store(STREAM_DB_TABLE).unwrap_throw();
    let item = store
        .get_owned(&serde_wasm_bindgen::to_value(&stream_id).unwrap_throw())
        .unwrap_throw()
        .await
        .unwrap_throw();
    let _ = tx;
    if item.is_some() {
        let stream: Stream =
            serde_wasm_bindgen::from_value(item.unwrap_throw()).unwrap_throw();
        let parse_json = to_string(&stream);
        if parse_json.is_ok() {
            return parse_json.unwrap_throw();
        }
    }
    return String::from("null");
}

#[wasm_bindgen]
pub async fn get_streams(stream_name: &str) -> String {
    let db = get_index_db().await.unwrap_throw();
    let tx = db.transaction_on_one(STREAM_DB_TABLE).unwrap_throw();
    let table = tx.object_store(STREAM_DB_TABLE).unwrap();
    let store = table.index("streamName").unwrap();
    let query = store
        .get_all_with_key_owned(stream_name)
        .unwrap()
        .await
        .unwrap();
    let streams: Vec<Stream> =
        serde_wasm_bindgen::from_value(JsValue::from(query)).unwrap();
    let _ = tx;
    let parse_json = to_string(&streams);
    if parse_json.is_ok() {
        return parse_json.unwrap_throw();
    }
    return String::from("null");
}
