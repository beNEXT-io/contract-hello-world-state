/*
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use aws_sdk_dynamodb::types::AttributeValue;
use chrono::Utc;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use lib::org_accordproject_helloworldstate::*;
use serde::{Deserialize, Serialize};
use utils::{add_data_to_database, add_state_to_database, get_data, increment_counter};

mod utils;

#[derive(Deserialize, Serialize, Debug)]
pub enum RequestType {
    MyRequest(MyRequest),
    HelloWorldClause(HelloWorldClause),
    // Add other request types here
}

#[derive(Deserialize, Serialize, Debug)]
pub enum ResponseType {
    MyResponse(MyResponse),
    HelloWorldClause(HelloWorldClause),
    // Add other response types here
}

#[derive(Deserialize, Serialize, Debug)]
struct Request {
    request: RequestType,
}

//
// Clause Function
//
// Function to handle the `MyRequest` clause
//
async fn handle_my_request(
    my_request: MyRequest,
) -> Result<MyResponse, Box<dyn std::error::Error>> {
    //
    // Increment the `{state}` counter held in DynamoDB.
    //
    let counter = match increment_counter().await {
        Ok(counter) => counter,
        Err(err) => {
            println!("Failed to increment counter: {}", err);
            return Err(err);
        }
    };

    //
    // Get the `{data}` from DynamoDB
    //
    let result = get_data("data").await;

    //
    // Generate the response depending on the result of the DynamoDB query
    //
    match result {
        Ok(Some(item)) => {
            // Extract the item from the response, if present
            if let Some(AttributeValue::S(name)) = item.get("name") {
                return Ok(MyResponse {
                    _class: my_request._class,
                    output: format!(
                        "Hello {} - {} - counter: {}",
                        name, my_request.input, counter
                    ),
                    _timestamp: Utc::now(),
                });
            }
            Err("Item not found or 'name' field missing".into())
        }
        Ok(None) => Err("Item not found".into()),
        Err(error) => {
            println!("Error: {:?}", error);
            Err(format!("AWS SDK error: {:?}", error).into())
        }
    }
}

//
// Constructor
//
// The constructor takes in the `{data}` payload and populates the DynamoDB database.
// The constructor also initiates the `{state}` of the agreement.
//
async fn new(
    hello_world_clause: HelloWorldClause,
) -> Result<HelloWorldClause, Box<dyn std::error::Error>> {
    add_data_to_database(&hello_world_clause)
        .await
        .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
    add_state_to_database(&hello_world_clause)
        .await
        .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;

    Ok(HelloWorldClause {
        _class: hello_world_clause._class,
        clause_id: hello_world_clause.clause_id,
        _identifier: hello_world_clause._identifier,
        name: hello_world_clause.name,
    })
}

//
// Main Function Handler
//
// This is the function that handles all incoming requests and determines which clause function to call.
//
async fn function_handler(event: LambdaEvent<Request>) -> Result<ResponseType, Error> {
    let response = match event.payload.request {
        RequestType::MyRequest(my_request) => {
            let my_response = handle_my_request(my_request)
                .await
                .map_err(|e| lambda_runtime::Error::from(format!("Error: {:?}", e)))?;
            ResponseType::MyResponse(my_response)
        }
        RequestType::HelloWorldClause(hello_world_clause) => {
            let clause = new(hello_world_clause)
                .await
                .map_err(|e| lambda_runtime::Error::from(format!("Error: {:?}", e)))?;
            ResponseType::HelloWorldClause(clause)
        }
    };

    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
