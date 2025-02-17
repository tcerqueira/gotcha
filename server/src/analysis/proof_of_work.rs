use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;

#[derive(Debug, Serialize, Deserialize)]
pub struct PowChallenge {
    pub nonce: u32,
    pub difficulty: u16,
    pub timestamp: i64,
}

impl PowChallenge {
    pub fn gen(difficulty: u16) -> Self {
        Self {
            nonce: rand::thread_rng().gen::<u32>(),
            difficulty,
            timestamp: OffsetDateTime::now_utc().unix_timestamp(),
        }
    }

    pub fn verify_solution(&self, solution: u32) -> bool {
        if self.difficulty == 0 || self.difficulty > 32 {
            return false;
        }

        let hash = self.hash_solution(solution);
        Self::is_solution(&hash, self.difficulty)
    }

    pub fn hash_solution(&self, solution: u32) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.nonce.to_be_bytes());
        hasher.update(self.difficulty.to_be_bytes());
        hasher.update(self.timestamp.to_be_bytes());
        hasher.update(solution.to_be_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn solve(&self) -> u32 {
        let mut solution = 0;
        loop {
            let hash = self.hash_solution(solution);
            if Self::is_solution(&hash, self.difficulty) {
                return solution;
            }
            solution += 1;
        }
    }

    fn is_solution(hash: &str, difficulty: u16) -> bool {
        let prefix = "0".repeat(difficulty as usize);
        hash.starts_with(&prefix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn successful_verify_solution() {
        let challenge = PowChallenge::gen(4);

        let solution = challenge.solve();

        let result = challenge.verify_solution(solution);
        assert!(result);
    }

    #[test]
    fn failed_verify_solution() {
        let challenge = PowChallenge::gen(4);

        let solution = challenge.solve() - 1;

        let result = challenge.verify_solution(solution);
        assert!(!result);
    }

    #[test]
    #[ignore = "useful for manually test values"]
    fn verify_specific_solution() {
        let challenge = PowChallenge { nonce: 4077096492, difficulty: 4, timestamp: 1739555092 };

        let solution = 13062;
        let hash = challenge.hash_solution(solution);
        eprintln!("{hash}");

        let actual_solution = challenge.solve();
        eprintln!("{actual_solution}");

        let result = challenge.verify_solution(solution);
        assert!(result);
    }
}
