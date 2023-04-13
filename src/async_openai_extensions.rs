// use async_openai::types::ChatCompletionResponseMessage;
// use serde::Serialize;
// use std::collections::BTreeMap;

// #[derive(Clone, PartialEq)]
// pub enum OpenAIChatCompletionMessage {
//     //Request(ChatCompletionRequestMessage),
//     Response(ChatCompletionResponseMessage),
// }

// // Implement the Serialize trait for ChatCompletionResponseMessageWrapper
// impl Serialize for OpenAIChatCompletionMessage {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         let mut map = BTreeMap::new();

//         match self {
//             // OpenAIChatCompletionMessage::Request(request_message) => {
//             //     map.insert("role", serde_json::to_value(&request_message.role).unwrap());
//             //     map.insert("content", serde_json::to_value(&request_message.content).unwrap());
//             // },
//             OpenAIChatCompletionMessage::Response(response_message) => {
//                 map.insert("role", serde_json::to_value(&response_message.role).unwrap());
//                 map.insert("content", serde_json::to_value(&response_message.content).unwrap());
//             }
//         }

//         map.serialize(serializer)
//     }
// }