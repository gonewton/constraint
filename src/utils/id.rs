//! Hash-based ID generation utilities

use sha2::{Digest, Sha256};

/// ID generation utility for constraints
pub struct IdGenerator;

impl IdGenerator {
    /// Create a new ID generator
    pub fn new() -> Self {
        Self
    }

    /// Generate a deterministic ID from constraint content
    pub fn generate(&mut self, text: &str, category: &str, constraint_type: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        hasher.update(category.as_bytes());
        hasher.update(constraint_type.as_bytes());

        let hash = hasher.finalize();

        // Take first 3 bytes (24 bits) and encode as base36, ensuring exactly 6 characters
        let suffix = Self::encode_base36_6chars(&hash[..3]);
        format!("nt-{}", suffix)
    }

    /// Validate ID format
    pub fn validate(id: &str) -> bool {
        // Validate format: nt-<base36-suffix> where suffix is exactly 6 characters
        let re = regex::Regex::new(r"^nt-[0-9a-z]{6}$").unwrap();
        re.is_match(id)
    }

    /// Convert 3 bytes to exactly 6 base36 characters with padding
    fn encode_base36_6chars(bytes: &[u8]) -> String {
        // Convert 3 bytes (24 bits) to u32
        let num = u32::from_be_bytes([0, bytes[0], bytes[1], bytes[2]]);
        let mut result = String::with_capacity(6);
        let chars: Vec<char> = "0123456789abcdefghijklmnopqrstuvwxyz".chars().collect();

        // Encode to base36, ensuring exactly 6 characters (pad with leading zeros)
        let mut temp_num = num;
        for _ in 0..6 {
            result.push(chars[(temp_num % 36) as usize]);
            temp_num /= 36;
        }

        result.chars().rev().collect::<String>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_generation_deterministic() {
        let mut generator = IdGenerator::new();

        let id1 = generator.generate("test text", "category", "MUST");
        let id2 = generator.generate("test text", "category", "MUST");

        // Different instances should generate same ID for same inputs
        assert_eq!(id1, id2);
        assert!(IdGenerator::validate(&id1));
    }

    #[test]
    fn test_id_validation() {
        assert!(IdGenerator::validate("nt-a1b2c3"));
        assert!(IdGenerator::validate("nt-012345"));
        assert!(IdGenerator::validate("nt-abcdef"));

        assert!(!IdGenerator::validate("invalid"));
        assert!(!IdGenerator::validate("nt-12345")); // too short
        assert!(!IdGenerator::validate("nt-1234567")); // too long
        assert!(!IdGenerator::validate("xx-123456")); // wrong prefix
    }

    #[test]
    fn test_collision_handling() {
        let mut generator = IdGenerator::new();

        // Generate first ID
        let id1 = generator.generate("text1", "cat", "MUST");

        // Generate another ID with different content
        let id2 = generator.generate("text2", "cat", "MUST");

        // They should be different
        assert_ne!(id1, id2);

        // Both should be valid
        assert!(IdGenerator::validate(&id1));
        assert!(IdGenerator::validate(&id2));
    }

    #[test]
    fn test_base36_encoding() {
        let bytes = [0x12, 0x34, 0x56];
        let encoded = IdGenerator::encode_base36_6chars(&bytes);

        assert_eq!(encoded.len(), 6);
        assert!(encoded
            .chars()
            .all(|c| c.is_alphanumeric() && c.is_lowercase() || c.is_ascii_digit()));
    }
}
