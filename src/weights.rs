use crate::config::TestCase;

/// Calcula cuántos requests corresponden a cada test según sus pesos
/// total_requests se distribuye proporcionalmente
pub fn distribute_requests(tests: &[TestCase], total_requests: u64) -> Vec<(usize, u64)> {
    let total_weight: f64 = tests.iter().map(|t| t.weight).sum();
    if total_weight == 0.0 {
        return vec![];
    }

    let mut distribution: Vec<(usize, u64)> = tests
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let n = ((t.weight / total_weight) * total_requests as f64).round() as u64;
            (i, n.max(1))
        })
        .collect();

    // Ajuste para que la suma sea exactamente total_requests
    let current_sum: u64 = distribution.iter().map(|(_, n)| n).sum();
    if current_sum != total_requests && !distribution.is_empty() {
        let diff = total_requests as i64 - current_sum as i64;
        let last = distribution.last_mut().unwrap();
        last.1 = (last.1 as i64 + diff).max(1) as u64;
    }

    distribution
}

/// Prepara copias de TestCase con el número de requests calculado por peso
pub fn apply_weights(tests: &[TestCase], total_requests: u64) -> Vec<TestCase> {
    let distribution = distribute_requests(tests, total_requests);
    distribution
        .into_iter()
        .map(|(i, n)| {
            let mut t = tests[i].clone();
            t.requests = Some(n);
            t.duration = None; // aseguramos que se use -n y no -z
            t
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TestCase;

    fn make_test(name: &str, weight: f64) -> TestCase {
        TestCase {
            name: name.to_string(),
            description: None,
            method: "GET".to_string(),
            url: "http://localhost".to_string(),
            query_params: Default::default(),
            headers: Default::default(),
            body: None,
            body_file: None,
            requests: None,
            connections: None,
            duration: None,
            query_per_second: None,
            timeout: None,
            weight,
            tags: vec![],
            http2: None,
            disable_keepalive: None,
            insecure: None,
        }
    }

    #[test]
    fn distribution_sums_to_total() {
        let tests = vec![
            make_test("a", 70.0),
            make_test("b", 30.0),
        ];
        let dist = distribute_requests(&tests, 1000);
        let total: u64 = dist.iter().map(|(_, n)| n).sum();
        assert_eq!(total, 1000);
    }

    #[test]
    fn distribution_proportional() {
        let tests = vec![
            make_test("a", 1.0),
            make_test("b", 1.0),
        ];
        let dist = distribute_requests(&tests, 100);
        assert_eq!(dist[0].1, 50);
        assert_eq!(dist[1].1, 50);
    }
}
