use rand::Rng;
use sha2::{Digest, Sha256};
use thiserror::Error;
use time::OffsetDateTime;

#[derive(Debug, Error)]
pub enum PowError {
    #[error("Invalid difficulty")]
    InvalidDifficulty,
    #[error("Solution doesn't meet difficulty requirement")]
    DifficultyNotMet,
}

#[derive(Debug)]
pub struct PowChallenge {
    pub nonce: u64,
    pub difficulty: u16,
    pub timestamp: i64,
}

impl PowChallenge {
    pub fn gen(difficulty: u16) -> Self {
        Self {
            nonce: rand::thread_rng().gen::<u64>(),
            difficulty,
            timestamp: OffsetDateTime::now_utc().unix_timestamp(),
        }
    }

    pub fn verify_solution(&self, solution: u64) -> Result<bool, PowError> {
        if self.difficulty == 0 || self.difficulty > 32 {
            return Err(PowError::InvalidDifficulty);
        }

        let hash = self.hash_solution(solution);
        if !Self::is_solution(&hash, self.difficulty) {
            return Err(PowError::DifficultyNotMet);
        }

        Ok(true)
    }

    pub fn hash_solution(&self, solution: u64) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.nonce.to_le_bytes());
        hasher.update(self.difficulty.to_le_bytes());
        hasher.update(self.timestamp.to_le_bytes());
        hasher.update(solution.to_le_bytes());
        format!("{:x}", hasher.finalize())
    }

    #[cfg(test)]
    fn solve(&self) -> u64 {
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
        let challenge = PowChallenge::gen(5);

        let solution = challenge.solve();

        let result = challenge.verify_solution(solution);
        assert!(result.is_ok());
    }

    #[test]
    fn failed_verify_solution() {
        let challenge = PowChallenge::gen(5);

        let solution = challenge.solve() - 1;

        let result = challenge.verify_solution(solution);
        assert!(result.is_err());
    }
}
