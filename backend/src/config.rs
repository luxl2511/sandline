use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub host: String,
    pub port: u16,
    pub allowed_origins: Vec<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let allowed_origins_str = std::env::var("ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3000".to_string());

        let allowed_origins = allowed_origins_str
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        Ok(Self {
            database_url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgresql://dakar_user:dakar_pass_dev_only@localhost:5432/dakar_planner"
                    .to_string()
            }),
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()?,
            allowed_origins,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_config_loads_database_url() {
        // Test that database_url is loaded (either from .env or default)
        let config = Config::from_env().unwrap();
        assert!(
            !config.database_url.is_empty(),
            "DATABASE_URL should not be empty"
        );
        assert!(
            config.database_url.starts_with("postgresql://"),
            "DATABASE_URL should be a PostgreSQL URL"
        );
    }

    #[test]
    #[serial]
    fn test_config_parses_custom_database_url() {
        let test_url = "postgresql://custom:pass@example.com:5432/testdb";
        std::env::set_var("DATABASE_URL", test_url);
        let config = Config::from_env().unwrap();
        assert_eq!(config.database_url, test_url);
        std::env::remove_var("DATABASE_URL");
    }

    #[test]
    #[serial]
    fn test_config_uses_default_host() {
        std::env::remove_var("HOST");
        let config = Config::from_env().unwrap();
        assert_eq!(config.host, "0.0.0.0");
    }

    #[test]
    #[serial]
    fn test_config_parses_custom_host() {
        std::env::set_var("HOST", "127.0.0.1");
        let config = Config::from_env().unwrap();
        assert_eq!(config.host, "127.0.0.1");
        std::env::remove_var("HOST");
    }

    #[test]
    #[serial]
    fn test_config_uses_default_port() {
        std::env::remove_var("PORT");
        let config = Config::from_env().unwrap();
        assert_eq!(config.port, 8080);
    }

    #[test]
    #[serial]
    fn test_config_parses_custom_port() {
        std::env::set_var("PORT", "3000");
        let config = Config::from_env().unwrap();
        assert_eq!(config.port, 3000);
        std::env::remove_var("PORT");
    }

    #[test]
    #[serial]
    fn test_config_port_parse_error() {
        std::env::set_var("PORT", "not_a_number");
        let result = Config::from_env();
        assert!(result.is_err(), "Should fail to parse invalid port");
        std::env::remove_var("PORT");
    }

    #[test]
    #[serial]
    fn test_config_port_out_of_range() {
        std::env::set_var("PORT", "99999");
        let result = Config::from_env();
        assert!(result.is_err(), "Should fail for port out of u16 range");
        std::env::remove_var("PORT");
    }

    #[test]
    #[serial]
    fn test_config_default_allowed_origins() {
        std::env::remove_var("ALLOWED_ORIGINS");
        let config = Config::from_env().unwrap();
        assert_eq!(config.allowed_origins, vec!["http://localhost:3000"]);
    }

    #[test]
    #[serial]
    fn test_config_parses_single_allowed_origin() {
        std::env::set_var("ALLOWED_ORIGINS", "https://example.com");
        let config = Config::from_env().unwrap();
        assert_eq!(config.allowed_origins, vec!["https://example.com"]);
        std::env::remove_var("ALLOWED_ORIGINS");
    }

    #[test]
    #[serial]
    fn test_config_parses_multiple_allowed_origins() {
        std::env::set_var(
            "ALLOWED_ORIGINS",
            "https://example.com,https://app.example.com,*.vercel.app",
        );
        let config = Config::from_env().unwrap();
        assert_eq!(
            config.allowed_origins,
            vec![
                "https://example.com",
                "https://app.example.com",
                "*.vercel.app"
            ]
        );
        std::env::remove_var("ALLOWED_ORIGINS");
    }

    #[test]
    #[serial]
    fn test_config_trims_allowed_origins() {
        std::env::set_var(
            "ALLOWED_ORIGINS",
            "  https://example.com  ,  https://app.example.com  ",
        );
        let config = Config::from_env().unwrap();
        assert_eq!(
            config.allowed_origins,
            vec!["https://example.com", "https://app.example.com"]
        );
        std::env::remove_var("ALLOWED_ORIGINS");
    }
}
