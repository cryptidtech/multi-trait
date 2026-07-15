// SPDX-License-Identifier: Apache-2.0
//! Custom type implementation example
//!
//! This example demonstrates how to implement multitrait traits for your
//! custom types, including:
#![allow(
    clippy::items_after_statements,
    clippy::struct_field_names,
    clippy::unreadable_literal
)]
//! - Implementing `EncodeInto` for custom types
//! - Implementing `TryDecodeFrom` for custom types
//! - Implementing Null and `TryNull` for custom types
//! - Creating composable encoding/decoding for complex structures

use multi_trait::{EncodeInto, EncodeIntoBuffer, Null, TryDecodeFrom, TryNull};

fn main() {
    println!("=== Multitrait Custom Type Example ===\n");

    // Example 1: Simple newtype wrapper
    simple_newtype();

    // Example 2: Struct with multiple fields
    multi_field_struct();

    // Example 3: Implementing Null trait
    null_trait_example();

    // Example 4: Implementing TryNull trait
    try_null_trait_example();

    // Example 5: Complex nested structures
    nested_structures();
}

/// Example 1: Simple newtype wrapper
fn simple_newtype() {
    println!("1. Simple Newtype Wrapper");
    println!("-------------------------");

    // Define a newtype
    #[derive(Debug, PartialEq)]
    struct UserId(u64);

    // Implement EncodeInto
    impl EncodeInto for UserId {
        fn encode_into(&self) -> Vec<u8> {
            self.0.encode_into()
        }
    }

    // Implement TryDecodeFrom
    impl<'a> TryDecodeFrom<'a> for UserId {
        type Error = multi_trait::Error;

        fn try_decode_from(bytes: &'a [u8]) -> Result<(Self, &'a [u8]), Self::Error> {
            let (id, remaining) = u64::try_decode_from(bytes)?;
            Ok((Self(id), remaining))
        }
    }

    // Use the custom type
    let user_id = UserId(12345);
    println!("Original UserId: {user_id:?}");

    let encoded = user_id.encode_into();
    println!("Encoded: {encoded:?}");

    let (decoded, _) = UserId::try_decode_from(&encoded).unwrap();
    println!("Decoded: {decoded:?}");

    assert_eq!(user_id, decoded);

    println!();
}

/// Example 2: Struct with multiple fields
fn multi_field_struct() {
    println!("2. Struct with Multiple Fields");
    println!("-------------------------------");

    // Define a struct with multiple fields
    #[derive(Debug, PartialEq)]
    struct Person {
        id: u32,
        age: u8,
        score: u16,
    }

    // Implement EncodeInto - encode fields sequentially
    impl EncodeInto for Person {
        fn encode_into(&self) -> Vec<u8> {
            let mut buffer = Vec::new();
            self.id.encode_into_buffer(&mut buffer);
            self.age.encode_into_buffer(&mut buffer);
            self.score.encode_into_buffer(&mut buffer);
            buffer
        }
    }

    // Implement TryDecodeFrom - decode fields sequentially
    impl<'a> TryDecodeFrom<'a> for Person {
        type Error = multi_trait::Error;

        fn try_decode_from(bytes: &'a [u8]) -> Result<(Self, &'a [u8]), Self::Error> {
            let (id, remaining) = u32::try_decode_from(bytes)?;
            let (age, remaining) = u8::try_decode_from(remaining)?;
            let (score, remaining) = u16::try_decode_from(remaining)?;

            Ok((Self { id, age, score }, remaining))
        }
    }

    // Use the custom type
    let person = Person {
        id: 1001,
        age: 25,
        score: 9500,
    };
    println!("Original Person: {person:?}");

    let encoded = person.encode_into();
    println!("Encoded ({} bytes): {:?}", encoded.len(), encoded);

    let (decoded, remaining) = Person::try_decode_from(&encoded).unwrap();
    println!("Decoded: {decoded:?}");
    println!("Remaining bytes: {remaining:?}");

    assert_eq!(person, decoded);

    println!();
}

/// Example 3: Implementing Null trait
fn null_trait_example() {
    println!("3. Implementing Null Trait");
    println!("--------------------------");

    // Define a type that can have a null value
    #[derive(Debug, PartialEq)]
    struct SessionId(u64);

    impl Null for SessionId {
        fn null() -> Self {
            Self(0)
        }

        fn is_null(&self) -> bool {
            self.0 == 0
        }
    }

    // Create null and non-null values
    let null_session = SessionId::null();
    let valid_session = SessionId(98765);

    println!(
        "Null session: {:?}, is_null: {}",
        null_session,
        null_session.is_null()
    );
    println!(
        "Valid session: {:?}, is_null: {}",
        valid_session,
        valid_session.is_null()
    );

    assert!(null_session.is_null());
    assert!(!valid_session.is_null());

    println!();
}

/// Example 4: Implementing `TryNull` trait
fn try_null_trait_example() {
    println!("4. Implementing TryNull Trait");
    println!("------------------------------");

    // Define a type with validated null creation
    #[derive(Debug, PartialEq)]
    struct ValidatedToken(String);

    impl TryNull for ValidatedToken {
        type Error = &'static str;

        fn try_null() -> Result<Self, Self::Error> {
            // Perform validation even for null value
            Ok(Self(String::from("NULL_TOKEN")))
        }

        fn is_null(&self) -> bool {
            self.0 == "NULL_TOKEN"
        }
    }

    // Create null token
    match ValidatedToken::try_null() {
        Ok(token) => {
            println!("Created null token: {token:?}");
            println!("Is null: {}", token.is_null());
            assert!(token.is_null());
        }
        Err(e) => {
            println!("Failed to create null token: {e}");
        }
    }

    // Non-null token
    let valid_token = ValidatedToken(String::from("abc123"));
    println!(
        "Valid token: {:?}, is_null: {}",
        valid_token,
        valid_token.is_null()
    );
    assert!(!valid_token.is_null());

    println!();
}

/// Example 5: Complex nested structures
fn nested_structures() {
    println!("5. Complex Nested Structures");
    println!("-----------------------------");

    // Define nested structures
    #[derive(Debug, PartialEq)]
    struct Metadata {
        version: u8,
        flags: u16,
    }

    #[derive(Debug, PartialEq)]
    struct Message {
        metadata: Metadata,
        sender_id: u64,
        message_type: u8,
    }

    // Implement EncodeInto for Metadata
    impl EncodeInto for Metadata {
        fn encode_into(&self) -> Vec<u8> {
            let mut buffer = Vec::new();
            self.version.encode_into_buffer(&mut buffer);
            self.flags.encode_into_buffer(&mut buffer);
            buffer
        }
    }

    // Implement TryDecodeFrom for Metadata
    impl<'a> TryDecodeFrom<'a> for Metadata {
        type Error = multi_trait::Error;

        fn try_decode_from(bytes: &'a [u8]) -> Result<(Self, &'a [u8]), Self::Error> {
            let (version, remaining) = u8::try_decode_from(bytes)?;
            let (flags, remaining) = u16::try_decode_from(remaining)?;
            Ok((Self { version, flags }, remaining))
        }
    }

    // Implement EncodeInto for Message
    impl EncodeInto for Message {
        fn encode_into(&self) -> Vec<u8> {
            let mut buffer = Vec::new();
            // Encode nested struct first
            buffer.extend_from_slice(&self.metadata.encode_into());
            self.sender_id.encode_into_buffer(&mut buffer);
            self.message_type.encode_into_buffer(&mut buffer);
            buffer
        }
    }

    // Implement TryDecodeFrom for Message
    impl<'a> TryDecodeFrom<'a> for Message {
        type Error = multi_trait::Error;

        fn try_decode_from(bytes: &'a [u8]) -> Result<(Self, &'a [u8]), Self::Error> {
            let (metadata, remaining) = Metadata::try_decode_from(bytes)?;
            let (sender_id, remaining) = u64::try_decode_from(remaining)?;
            let (message_type, remaining) = u8::try_decode_from(remaining)?;

            Ok((
                Self {
                    metadata,
                    sender_id,
                    message_type,
                },
                remaining,
            ))
        }
    }

    // Use the nested structures
    let message = Message {
        metadata: Metadata {
            version: 1,
            flags: 0x0F00,
        },
        sender_id: 999888777,
        message_type: 42,
    };

    println!("Original message: {message:#?}");

    let encoded = message.encode_into();
    println!("Encoded ({} bytes): {:?}", encoded.len(), encoded);

    let (decoded, remaining) = Message::try_decode_from(&encoded).unwrap();
    println!("Decoded message: {decoded:#?}");
    println!("Remaining bytes: {remaining:?}");

    assert_eq!(message, decoded);

    println!();
}
