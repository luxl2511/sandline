use anyhow::{anyhow, Result};
use geo::algorithm::simplify::Simplify;
use geo::LineString;
use serde_json::Value;

/// Simplifies a MultiLineString geometry using Douglas-Peucker algorithm
///
/// Target: 100km route should have ~200-500 points max (not 10,000+)
/// Epsilon: 0.0001 degrees ≈ 11 meters tolerance
///
/// # Arguments
/// * `geometry` - GeoJSON MultiLineString as JSON Value
/// * `epsilon` - Simplification tolerance in degrees (default: 0.0001 ≈ 11m)
///
/// # Returns
/// Simplified GeoJSON MultiLineString
pub fn simplify_geometry(geometry: &Value, epsilon: f64) -> Result<Value> {
    // Parse GeoJSON MultiLineString
    let coords = geometry["coordinates"]
        .as_array()
        .ok_or_else(|| anyhow!("Invalid geometry: missing coordinates"))?;

    let mut simplified_coords = Vec::new();

    for line_coords in coords {
        let points: Vec<(f64, f64)> = line_coords
            .as_array()
            .ok_or_else(|| anyhow!("Invalid line coordinates"))?
            .iter()
            .filter_map(|p| {
                let arr = p.as_array()?;
                Some((arr[0].as_f64()?, arr[1].as_f64()?))
            })
            .collect();

        // Skip empty or single-point lines
        if points.len() < 2 {
            continue;
        }

        let linestring = LineString::from(points);
        let simplified = linestring.simplify(&epsilon);

        let simplified_points: Vec<Vec<f64>> = simplified
            .0
            .iter()
            .map(|coord| vec![coord.x, coord.y])
            .collect();

        simplified_coords.push(simplified_points);
    }

    Ok(serde_json::json!({
        "type": "MultiLineString",
        "coordinates": simplified_coords
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simplification() {
        let geometry = serde_json::json!({
            "type": "MultiLineString",
            "coordinates": [
                [
                    [-5.0, 20.0],
                    [-5.0001, 20.0001], // Very close point - should be removed
                    [-5.1, 20.1],
                    [-5.1001, 20.1001], // Very close point - should be removed
                    [-5.2, 20.2]
                ]
            ]
        });

        let simplified = simplify_geometry(&geometry, 0.0001).unwrap();
        let coords = simplified["coordinates"][0].as_array().unwrap();

        // Should reduce from 5 points to 3 points
        assert!(coords.len() < 5);
        assert!(coords.len() >= 2); // At least start and end
    }

    #[test]
    fn test_empty_geometry() {
        let geometry = serde_json::json!({
            "type": "MultiLineString",
            "coordinates": []
        });

        let simplified = simplify_geometry(&geometry, 0.0001).unwrap();
        assert_eq!(simplified["coordinates"].as_array().unwrap().len(), 0);
    }
}
