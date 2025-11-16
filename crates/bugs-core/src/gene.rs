use serde::{Deserialize, Serialize};

/// Gene types for genetic programming
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GeneType {
    Constant = 1,
    Sense = 2,
    Limit = 3,
    Compare = 4,
    Match = 5,
}

/// A gene in the genetic programming system
/// Genes form expression trees for decision-making
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gene {
    pub gene_type: GeneType,
    pub sense_index: usize,
    pub c1: i32,
    pub c2: i32,

    // Links for expression tree evaluation
    pub prod_index: Option<usize>,  // multiply by this gene's result
    pub sum_index: Option<usize>,   // add this gene's result
}

impl Gene {
    pub fn new_constant(value: i32) -> Self {
        Self {
            gene_type: GeneType::Constant,
            sense_index: 0,
            c1: value,
            c2: 0,
            prod_index: None,
            sum_index: None,
        }
    }

    pub fn new_sense(sense_index: usize) -> Self {
        Self {
            gene_type: GeneType::Sense,
            sense_index,
            c1: 0,
            c2: 0,
            prod_index: None,
            sum_index: None,
        }
    }

    pub fn new_limit(sense_index: usize, min: i32, max: i32) -> Self {
        Self {
            gene_type: GeneType::Limit,
            sense_index,
            c1: min,
            c2: max,
            prod_index: None,
            sum_index: None,
        }
    }

    pub fn new_compare(sense_index: usize, threshold: i32) -> Self {
        Self {
            gene_type: GeneType::Compare,
            sense_index,
            c1: threshold,
            c2: 0,
            prod_index: None,
            sum_index: None,
        }
    }

    pub fn new_match(sense_index: usize) -> Self {
        Self {
            gene_type: GeneType::Match,
            sense_index,
            c1: 0,
            c2: 0,
            prod_index: None,
            sum_index: None,
        }
    }

    /// Evaluate this gene given sense data and previously evaluated genes
    pub fn evaluate(&self, senses: &[i32], gene_values: &[f64]) -> f64 {
        let base_value = match self.gene_type {
            GeneType::Constant => self.c1 as f64,
            GeneType::Sense => {
                if self.sense_index < senses.len() {
                    senses[self.sense_index] as f64
                } else {
                    0.0
                }
            }
            GeneType::Limit => {
                if self.sense_index < senses.len() {
                    let val = senses[self.sense_index];
                    val.clamp(self.c1, self.c2) as f64
                } else {
                    0.0
                }
            }
            GeneType::Compare => {
                if self.sense_index < senses.len() {
                    if senses[self.sense_index] > self.c1 { 1.0 } else { 0.0 }
                } else {
                    0.0
                }
            }
            GeneType::Match => {
                if self.sense_index < senses.len() {
                    // Match function - returns similarity metric
                    senses[self.sense_index] as f64
                } else {
                    0.0
                }
            }
        };

        // Apply prod and sum operations: value = base_value * prod_value + sum_value
        let prod_value = if let Some(idx) = self.prod_index {
            gene_values[idx]
        } else {
            1.0
        };

        let sum_value = if let Some(idx) = self.sum_index {
            gene_values[idx]
        } else {
            0.0
        };

        base_value * prod_value + sum_value
    }
}

/// A chromosome - collection of genes for one decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chromosome {
    pub genes: Vec<Gene>,
    pub ethnicity: Ethnicity,
}

impl Chromosome {
    pub fn new() -> Self {
        Self {
            genes: Vec::new(),
            ethnicity: Ethnicity::default(),
        }
    }

    pub fn with_genes(genes: Vec<Gene>, ethnicity: Ethnicity) -> Self {
        Self { genes, ethnicity }
    }

    /// Evaluate all genes in this chromosome and return the final weight
    pub fn evaluate(&self, senses: &[i32]) -> f64 {
        if self.genes.is_empty() {
            return 0.0;
        }

        let mut gene_values = vec![0.0; self.genes.len()];

        // Evaluate genes in order
        for (i, gene) in self.genes.iter().enumerate() {
            gene_values[i] = gene.evaluate(senses, &gene_values);
        }

        // Return the last gene's value as the chromosome's output
        gene_values.last().copied().unwrap_or(0.0)
    }
}

impl Default for Chromosome {
    fn default() -> Self {
        Self::new()
    }
}

/// Ethnicity tracking for genetic lineage visualization
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Ethnicity {
    pub uid: u64,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Default for Ethnicity {
    fn default() -> Self {
        Self {
            uid: 0,
            r: 0,
            g: 0,
            b: 0,
        }
    }
}

impl Ethnicity {
    pub fn new(uid: u64, r: u8, g: u8, b: u8) -> Self {
        Self { uid, r, g, b }
    }

    /// Blend two ethnicities (for mating)
    pub fn blend(&self, other: &Ethnicity) -> Self {
        Self {
            uid: self.uid.max(other.uid),
            r: ((self.r as u16 + other.r as u16) / 2) as u8,
            g: ((self.g as u16 + other.g as u16) / 2) as u8,
            b: ((self.b as u16 + other.b as u16) / 2) as u8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_gene() {
        let gene = Gene::new_constant(42);
        let senses = vec![0; 10];
        let gene_values = vec![];
        assert_eq!(gene.evaluate(&senses, &gene_values), 42.0);
    }

    #[test]
    fn test_sense_gene() {
        let gene = Gene::new_sense(2);
        let senses = vec![10, 20, 30, 40];
        let gene_values = vec![];
        assert_eq!(gene.evaluate(&senses, &gene_values), 30.0);
    }

    #[test]
    fn test_limit_gene() {
        let gene = Gene::new_limit(0, 10, 50);
        let gene_values = vec![];

        let senses = vec![5];
        assert_eq!(gene.evaluate(&senses, &gene_values), 10.0); // clamped to min

        let senses = vec![100];
        assert_eq!(gene.evaluate(&senses, &gene_values), 50.0); // clamped to max

        let senses = vec![30];
        assert_eq!(gene.evaluate(&senses, &gene_values), 30.0); // within range
    }
}
