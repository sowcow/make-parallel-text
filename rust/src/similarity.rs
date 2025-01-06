use rust_bert::pipelines::sentence_embeddings::SentenceEmbeddingsBuilder;
use rust_bert::pipelines::sentence_embeddings::SentenceEmbeddingsModel;
use rust_bert::pipelines::sentence_embeddings::SentenceEmbeddingsModelType;
use tch::{Device, Kind, Tensor};

pub struct Similarity {
    model: SentenceEmbeddingsModel,
}

impl Similarity {
    pub fn new() -> Self {
        // (for my case Rus<>Srb only the latest worked, it seems too general to not use it)
        //
        //let model = SentenceEmbeddingsBuilder::remote(SentenceEmbeddingsModelType::AllMiniLmL6V2)
        //let model = SentenceEmbeddingsBuilder::remote(SentenceEmbeddingsModelType::SentenceT5Base)
        let model = SentenceEmbeddingsBuilder::remote(
            SentenceEmbeddingsModelType::DistiluseBaseMultilingualCased,
        )
        .with_device(Device::cuda_if_available())
        .create_model()
        .unwrap();

        Similarity { model }
    }

    pub fn get(&self, a: &str, b: &str) -> f32 {
        let sentences = vec![a, b];
        let embeddings: Vec<Vec<f32>> = self.model.encode(&sentences).unwrap();
        cosine_similarity(&embeddings[0], &embeddings[1])
    }

    pub fn get_many(&self, xs: &[&str]) -> Vec<Vec<f32>> {
        let embeddings: Vec<Vec<f32>> = self.model.encode(&xs).unwrap();
        let tensor = cosine_similarity_matrix(&embeddings);

        let shape = tensor.size();
        let flat = tensor.flatten(0, -1);
        let v = Vec::<f32>::try_from(flat).expect("wrong type of tensor");

        let rows = shape[0] as usize;
        let cols = shape[1] as usize;
        let nested_data: Vec<Vec<f32>> = v.chunks(cols).map(|chunk| chunk.to_vec()).collect();

        nested_data
    }
}

fn cosine_similarity_matrix(embeddings: &[Vec<f32>]) -> tch::Tensor {
    let n = embeddings.len();
    if n == 0 {
        panic!("No embeddings provided!");
    }

    let d = embeddings[0].len();
    let mut data = Vec::with_capacity(n * d);

    // Flatten the Vec<Vec<f32>> into a single Vec<f32>
    for embed in embeddings {
        if embed.len() != d {
            panic!("All embeddings must have the same dimension.");
        }
        data.extend_from_slice(embed);
    }

    // Create an N x D Tensor from the flattened data
    let emb_tensor = Tensor::from_slice(&data).reshape(&[n as i64, d as i64]);

    // L2-normalize each row (each embedding)
    // norm: shape [N, 1]
    let norm = emb_tensor
        .square()
        //.pow_scalar(2.0)
        .sum_dim_intlist([1].as_ref(), true, Kind::Float)
        .sqrt();
    let normalized = &emb_tensor / &norm;

    // Cosine similarity is just the matrix multiplication
    // of normalized with its transpose
    let similarity = normalized.matmul(&normalized.transpose(0, 1));
    similarity
}

fn cosine_similarity(vec1: &Vec<f32>, vec2: &Vec<f32>) -> f32 {
    use nalgebra::DVector;
    let v1 = DVector::from_vec(vec1.clone());
    let v2 = DVector::from_vec(vec2.clone());
    v1.dot(&v2) / (v1.norm() * v2.norm())
}
