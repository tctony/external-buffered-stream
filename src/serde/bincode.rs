use bincode::{config, decode_from_slice, encode_to_vec};
pub use bincode::{Decode, Encode};

use crate::Error;

use super::ExternalBufferSerde;

impl<T> ExternalBufferSerde for T
where
    T: Encode + Decode<()>,
{
    fn into_external_buffer(self) -> Result<Vec<u8>, Error> {
        Ok(encode_to_vec(self, config::standard())?)
    }

    fn from_external_buffer(buffer: &[u8]) -> Result<T, Error> {
        Ok(decode_from_slice(buffer, config::standard()).map(|(u, _)| u)?)
    }
}

#[cfg(test)]
mod tests {
    use super::ExternalBufferSerde;
    use bincode::{Decode, Encode};

    #[derive(Debug, Clone, PartialEq, Encode, Decode)]
    struct TestStruct {
        id: u32,
        name: String,
        active: bool,
    }

    #[derive(Debug, Clone, PartialEq, Encode, Decode)]
    struct SimpleStruct {
        value: i32,
    }

    #[test]
    fn test_basic_encode_decode() {
        let original = TestStruct {
            id: 42,
            name: "test".to_string(),
            active: true,
        };

        // Test encoding
        let encoded = original
            .clone()
            .into_external_buffer()
            .expect("Failed to encode");
        assert!(!encoded.is_empty(), "Encoded data should not be empty");

        // Test decoding
        let decoded = TestStruct::from_external_buffer(&encoded).expect("Failed to decode");
        assert_eq!(original, decoded, "Decoded value should match original");
    }

    #[test]
    fn test_simple_types() {
        // Test with integer
        let original_int = 12345i32;
        let encoded = original_int
            .into_external_buffer()
            .expect("Failed to encode i32");
        let decoded = i32::from_external_buffer(&encoded).expect("Failed to decode i32");
        assert_eq!(original_int, decoded);

        // Test with string
        let original_string = "Hello, World!".to_string();
        let encoded = original_string
            .clone()
            .into_external_buffer()
            .expect("Failed to encode String");
        let decoded = String::from_external_buffer(&encoded).expect("Failed to decode String");
        assert_eq!(original_string, decoded);

        // Test with boolean
        let original_bool = true;
        let encoded = original_bool
            .into_external_buffer()
            .expect("Failed to encode bool");
        let decoded = bool::from_external_buffer(&encoded).expect("Failed to decode bool");
        assert_eq!(original_bool, decoded);
    }

    #[test]
    fn test_empty_string() {
        let original = String::new();
        let encoded = original
            .clone()
            .into_external_buffer()
            .expect("Failed to encode empty string");
        let decoded =
            String::from_external_buffer(&encoded).expect("Failed to decode empty string");
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_zero_values() {
        let original = SimpleStruct { value: 0 };
        let encoded = original
            .clone()
            .into_external_buffer()
            .expect("Failed to encode zero value");
        let decoded =
            SimpleStruct::from_external_buffer(&encoded).expect("Failed to decode zero value");
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_negative_values() {
        let original = SimpleStruct { value: -42 };
        let encoded = original
            .clone()
            .into_external_buffer()
            .expect("Failed to encode negative value");
        let decoded =
            SimpleStruct::from_external_buffer(&encoded).expect("Failed to decode negative value");
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_large_struct() {
        let original = TestStruct {
            id: u32::MAX,
            name: "a".repeat(1000), // Large string
            active: false,
        };
        let encoded = original
            .clone()
            .into_external_buffer()
            .expect("Failed to encode large struct");
        let decoded =
            TestStruct::from_external_buffer(&encoded).expect("Failed to decode large struct");
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_decode_invalid_data() {
        // Test with completely invalid data
        let invalid_data = vec![0xFF, 0xFF, 0xFF, 0xFF];
        let result: Result<TestStruct, _> = TestStruct::from_external_buffer(&invalid_data);
        assert!(result.is_err(), "Should fail to decode invalid data");

        // Verify it's a decode error
        match result {
            Err(crate::Error::DecodeError(_)) => {}
            _ => panic!("Expected DecodeError"),
        }
    }

    #[test]
    fn test_decode_empty_data() {
        let empty_data = vec![];
        let result: Result<i32, _> = i32::from_external_buffer(&empty_data);
        assert!(result.is_err(), "Should fail to decode empty data");
    }

    #[test]
    fn test_decode_truncated_data() {
        // Encode a struct first
        let original = TestStruct {
            id: 42,
            name: "test".to_string(),
            active: true,
        };
        let mut encoded = original.into_external_buffer().expect("Failed to encode");

        // Truncate the data
        encoded.truncate(encoded.len() / 2);

        let result: Result<TestStruct, _> = TestStruct::from_external_buffer(&encoded);
        assert!(result.is_err(), "Should fail to decode truncated data");
    }

    #[test]
    fn test_roundtrip_multiple_times() {
        let mut current = TestStruct {
            id: 1,
            name: "initial".to_string(),
            active: true,
        };

        // Perform multiple encode/decode cycles
        for i in 0..10 {
            let encoded = current
                .clone()
                .into_external_buffer()
                .expect("Failed to encode in cycle");
            current =
                TestStruct::from_external_buffer(&encoded).expect("Failed to decode in cycle");
            current.id = i + 2; // Modify for next iteration
        }

        assert_eq!(current.id, 11);
        assert_eq!(current.name, "initial");
        assert_eq!(current.active, true);
    }

    #[test]
    fn test_vec_serialization() {
        let original = vec![1, 2, 3, 4, 5];
        let encoded = original
            .clone()
            .into_external_buffer()
            .expect("Failed to encode vec");
        let decoded: Vec<i32> = Vec::from_external_buffer(&encoded).expect("Failed to decode vec");
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_option_serialization() {
        // Test Some value
        let original_some = Some(42i32);
        let encoded = original_some
            .into_external_buffer()
            .expect("Failed to encode Some");
        let decoded: Option<i32> =
            Option::from_external_buffer(&encoded).expect("Failed to decode Some");
        assert_eq!(original_some, decoded);

        // Test None value
        let original_none: Option<i32> = None;
        let encoded = original_none
            .into_external_buffer()
            .expect("Failed to encode None");
        let decoded: Option<i32> =
            Option::from_external_buffer(&encoded).expect("Failed to decode None");
        assert_eq!(original_none, decoded);
    }

    #[test]
    fn test_tuple_serialization() {
        let original = (42i32, "hello".to_string(), true);
        let encoded = original
            .clone()
            .into_external_buffer()
            .expect("Failed to encode tuple");
        let decoded: (i32, String, bool) =
            <(i32, String, bool)>::from_external_buffer(&encoded).expect("Failed to decode tuple");
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_nested_struct_serialization() {
        #[derive(Debug, Clone, PartialEq, Encode, Decode)]
        struct NestedStruct {
            inner: TestStruct,
            count: u64,
            values: Vec<i32>,
        }

        let original = NestedStruct {
            inner: TestStruct {
                id: 123,
                name: "nested".to_string(),
                active: true,
            },
            count: 456,
            values: vec![1, 2, 3, 4, 5],
        };

        let encoded = original
            .clone()
            .into_external_buffer()
            .expect("Failed to encode nested struct");
        let decoded: NestedStruct =
            NestedStruct::from_external_buffer(&encoded).expect("Failed to decode nested struct");
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_empty_vec_serialization() {
        let original: Vec<i32> = vec![];
        let encoded = original
            .clone()
            .into_external_buffer()
            .expect("Failed to encode empty vec");
        let decoded: Vec<i32> =
            Vec::from_external_buffer(&encoded).expect("Failed to decode empty vec");
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_large_vec_serialization() {
        let original: Vec<u64> = (0..10000).collect();
        let encoded = original
            .clone()
            .into_external_buffer()
            .expect("Failed to encode large vec");
        let decoded: Vec<u64> =
            Vec::from_external_buffer(&encoded).expect("Failed to decode large vec");
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_enum_serialization() {
        #[derive(Debug, Clone, PartialEq, Encode, Decode)]
        enum TestEnum {
            Variant1,
            Variant2(i32),
            Variant3 { name: String, value: u32 },
        }

        // Test simple variant
        let original1 = TestEnum::Variant1;
        let encoded1 = original1
            .clone()
            .into_external_buffer()
            .expect("Failed to encode enum variant1");
        let decoded1: TestEnum =
            TestEnum::from_external_buffer(&encoded1).expect("Failed to decode enum variant1");
        assert_eq!(original1, decoded1);

        // Test variant with data
        let original2 = TestEnum::Variant2(42);
        let encoded2 = original2
            .clone()
            .into_external_buffer()
            .expect("Failed to encode enum variant2");
        let decoded2: TestEnum =
            TestEnum::from_external_buffer(&encoded2).expect("Failed to decode enum variant2");
        assert_eq!(original2, decoded2);

        // Test variant with struct-like data
        let original3 = TestEnum::Variant3 {
            name: "test".to_string(),
            value: 123,
        };
        let encoded3 = original3
            .clone()
            .into_external_buffer()
            .expect("Failed to encode enum variant3");
        let decoded3: TestEnum =
            TestEnum::from_external_buffer(&encoded3).expect("Failed to decode enum variant3");
        assert_eq!(original3, decoded3);
    }

    #[test]
    fn test_buffer_size_consistency() {
        let test_data = TestStruct {
            id: 42,
            name: "consistency_test".to_string(),
            active: true,
        };

        // Encode the same data multiple times and verify buffer size is consistent
        let encoded1 = test_data
            .clone()
            .into_external_buffer()
            .expect("First encoding failed");
        let encoded2 = test_data
            .clone()
            .into_external_buffer()
            .expect("Second encoding failed");
        let encoded3 = test_data
            .clone()
            .into_external_buffer()
            .expect("Third encoding failed");

        assert_eq!(
            encoded1.len(),
            encoded2.len(),
            "Buffer sizes should be consistent"
        );
        assert_eq!(
            encoded2.len(),
            encoded3.len(),
            "Buffer sizes should be consistent"
        );
        assert_eq!(encoded1, encoded2, "Encoded buffers should be identical");
        assert_eq!(encoded2, encoded3, "Encoded buffers should be identical");
    }

    #[test]
    fn test_extreme_values() {
        // Test with extreme integer values
        let max_u32 = u32::MAX;
        let encoded = max_u32
            .into_external_buffer()
            .expect("Failed to encode max u32");
        let decoded = u32::from_external_buffer(&encoded).expect("Failed to decode max u32");
        assert_eq!(max_u32, decoded);

        let min_i32 = i32::MIN;
        let encoded = min_i32
            .into_external_buffer()
            .expect("Failed to encode min i32");
        let decoded = i32::from_external_buffer(&encoded).expect("Failed to decode min i32");
        assert_eq!(min_i32, decoded);

        let max_i64 = i64::MAX;
        let encoded = max_i64
            .into_external_buffer()
            .expect("Failed to encode max i64");
        let decoded = i64::from_external_buffer(&encoded).expect("Failed to decode max i64");
        assert_eq!(max_i64, decoded);
    }
}
