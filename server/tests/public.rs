use std::sync::LazyLock;

use reqwest::Client;

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

mod verify_site {
    use std::net::SocketAddr;

    use gotcha_server::{
        response_token::{self, ResponseClaims},
        routes::public::{ErrorCodes, VerificationResponse},
    };
    use reqwest::StatusCode;

    use super::*;

    #[gotcha_server_macros::integration_test]
    async fn sucessful_challenge(server: TestContext) -> anyhow::Result<()> {
        let port = server.port();
        let api_secret = server.db_api_secret().await;
        let enc_key = server.db_enconding_key().await;

        let token = response_token::encode(
            ResponseClaims {
                success: true,
                authority: SocketAddr::new([127, 0, 0, 1].into(), port),
            },
            &enc_key,
        )?;

        let response = HTTP_CLIENT
            .post(format!("http://localhost:{port}/api/siteverify"))
            .form(&[("secret", &api_secret), ("response", &token)])
            .send()
            .await?;
        assert_eq!(response.status(), StatusCode::OK);

        let verification: VerificationResponse = response.json().await?;
        assert!(verification.success);
        assert_eq!(verification.error_codes, None);

        Ok(())
    }

    #[gotcha_server_macros::integration_test]
    async fn failed_challenge(server: TestContext) -> anyhow::Result<()> {
        let port = server.port();
        let api_secret = server.db_api_secret().await;
        let enc_key = server.db_enconding_key().await;

        let token = response_token::encode(
            ResponseClaims {
                success: false,
                authority: SocketAddr::new([127, 0, 0, 1].into(), port),
            },
            &enc_key,
        )?;

        let response = HTTP_CLIENT
            .post(format!("http://localhost:{port}/api/siteverify"))
            .form(&[("secret", &api_secret), ("response", &token)])
            .send()
            .await?;
        assert_eq!(response.status(), StatusCode::OK);

        let verification: VerificationResponse = response.json().await?;
        assert!(!verification.success);
        assert_eq!(verification.error_codes, None);

        Ok(())
    }

    #[gotcha_server_macros::integration_test]
    async fn missing_secret(server: TestContext) -> anyhow::Result<()> {
        let port = server.port();
        let enc_key = server.db_enconding_key().await;

        let token = response_token::encode(
            ResponseClaims {
                success: true,
                authority: SocketAddr::new([127, 0, 0, 1].into(), port),
            },
            &enc_key,
        )?;

        let response = HTTP_CLIENT
            .post(format!("http://localhost:{port}/api/siteverify"))
            .form(&[("response", &token)])
            .send()
            .await?;
        assert_eq!(response.status(), StatusCode::OK);

        let verification: VerificationResponse = response.json().await?;
        assert!(!verification.success);
        assert!(verification
            .error_codes
            .expect("must have error codes")
            .contains(&ErrorCodes::MissingInputSecret));

        Ok(())
    }

    #[gotcha_server_macros::integration_test]
    async fn missing_response(server: TestContext) -> anyhow::Result<()> {
        let port = server.port();
        let api_secret = server.db_api_secret().await;

        let response = HTTP_CLIENT
            .post(format!("http://localhost:{port}/api/siteverify"))
            .form(&[("secret", api_secret)])
            .send()
            .await?;
        assert_eq!(response.status(), StatusCode::OK);

        let verification: VerificationResponse = response.json().await?;
        assert!(!verification.success);
        assert!(verification
            .error_codes
            .expect("must have error codes")
            .contains(&ErrorCodes::MissingInputResponse));

        Ok(())
    }

    #[gotcha_server_macros::integration_test]
    async fn missing_secret_and_response(server: TestContext) -> anyhow::Result<()> {
        let port = server.port();

        let response = HTTP_CLIENT
            .post(format!("http://localhost:{port}/api/siteverify"))
            .form::<[(&str, &str)]>(&[])
            .send()
            .await?;
        assert_eq!(response.status(), StatusCode::OK);

        let verification: VerificationResponse = response.json().await?;
        assert!(!verification.success);
        assert!(verification
            .error_codes
            .as_ref()
            .expect("must have error codes")
            .contains(&ErrorCodes::MissingInputSecret));
        assert!(verification
            .error_codes
            .as_ref()
            .expect("must have error codes")
            .contains(&ErrorCodes::MissingInputResponse));

        Ok(())
    }

    #[gotcha_server_macros::integration_test]
    async fn invalid_secret(server: TestContext) -> anyhow::Result<()> {
        let port = server.port();
        let enc_key = server.db_enconding_key().await;

        let token = response_token::encode(
            ResponseClaims {
                success: true,
                authority: SocketAddr::new([127, 0, 0, 1].into(), port),
            },
            &enc_key,
        )?;

        let invalid_secret = "AAABBBCC";
        let response = HTTP_CLIENT
            .post(format!("http://localhost:{port}/api/siteverify"))
            .form(&[("secret", invalid_secret), ("response", &token)])
            .send()
            .await?;
        assert_eq!(response.status(), StatusCode::OK);

        let verification: VerificationResponse = response.json().await?;
        assert!(!verification.success);
        assert!(verification
            .error_codes
            .expect("must have error codes")
            .contains(&ErrorCodes::InvalidInputSecret));

        Ok(())
    }

    #[gotcha_server_macros::integration_test]
    async fn bad_request(server: TestContext) -> anyhow::Result<()> {
        let port = server.port();

        let response = HTTP_CLIENT
            .post(format!("http://localhost:{port}/api/siteverify"))
            // .header("Content-Type", "application/x-www-form-urlencoded")
            .body("secret=api_key&response=response_token")
            .send()
            .await?;
        assert_eq!(response.status(), StatusCode::OK);

        let verification: VerificationResponse = response.json().await?;
        assert!(!verification.success);
        assert!(verification
            .error_codes
            .expect("must have error codes")
            .contains(&ErrorCodes::BadRequest));

        Ok(())
    }

    #[gotcha_server_macros::integration_test]
    async fn duplicate(_server: TestContext) -> anyhow::Result<()> {
        // TODO
        Ok(())
    }

    mod response {
        use std::time::Duration;

        use jsonwebtoken::{EncodingKey, Header};
        use response_token::{Claims, JWT_ALGORITHM};

        use super::*;

        #[gotcha_server_macros::integration_test]
        async fn expired_signature(server: TestContext) -> anyhow::Result<()> {
            let port = server.port();
            let api_secret = server.db_api_secret().await;
            let enc_key = server.db_enconding_key().await;

            let token = response_token::encode_with_timeout(
                Duration::from_secs(0),
                ResponseClaims {
                    success: true,
                    authority: SocketAddr::new([127, 0, 0, 1].into(), port),
                },
                &enc_key,
            )?;
            // expired by 1 second
            tokio::time::sleep(Duration::from_secs(1)).await;

            let response = HTTP_CLIENT
                .post(format!("http://localhost:{port}/api/siteverify"))
                .form(&[("secret", api_secret), ("response", token)])
                .send()
                .await?;
            assert_eq!(response.status(), StatusCode::OK);

            let verification: VerificationResponse = response.json().await?;
            assert!(!verification.success);
            assert!(verification
                .error_codes
                .expect("must have error codes")
                .contains(&ErrorCodes::TimeoutOrDuplicate));

            Ok(())
        }

        #[gotcha_server_macros::integration_test]
        async fn immature_signature(_server: TestContext) -> anyhow::Result<()> {
            // TODO
            Ok(())
        }

        #[gotcha_server_macros::integration_test]
        async fn invalid_token(server: TestContext) -> anyhow::Result<()> {
            let port = server.port();
            let api_secret = server.db_api_secret().await;

            let token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..eyJleHAiOjE3MzAzMDIyNDYsInN1Y2Nlc3MiOnRydWV9.9VBstXEca0JEPksQbMOEXdL_MxBvjiDgLbp0JnfsXMw";
            //                                                ^ extra dot
            let response = HTTP_CLIENT
                .post(format!("http://localhost:{port}/api/siteverify"))
                .form(&[("secret", api_secret.as_str()), ("response", token)])
                .send()
                .await?;
            assert_eq!(response.status(), StatusCode::OK);

            let verification: VerificationResponse = response.json().await?;
            assert!(!verification.success);
            assert!(verification
                .error_codes
                .expect("must have error codes")
                .contains(&ErrorCodes::InvalidInputResponse));

            Ok(())
        }

        #[gotcha_server_macros::integration_test]
        async fn invalid_signature(server: TestContext) -> anyhow::Result<()> {
            let port = server.port();
            let api_secret = server.db_api_secret().await;

            let token = jsonwebtoken::encode(
                &Header::new(JWT_ALGORITHM),
                &Claims::new(ResponseClaims {
                    success: true,
                    authority: SocketAddr::new([127, 0, 0, 1].into(), port),
                }),
                &EncodingKey::from_base64_secret(
                    "bXktd3Jvbmctc2VjcmV0", /* `my-wrong-secret` in base64 */
                )?,
            )?;

            let response = HTTP_CLIENT
                .post(format!("http://localhost:{port}/api/siteverify"))
                .form(&[("secret", api_secret), ("response", token)])
                .send()
                .await?;
            assert_eq!(response.status(), StatusCode::OK);

            let verification: VerificationResponse = response.json().await?;
            assert!(!verification.success);
            assert!(verification
                .error_codes
                .expect("must have error codes")
                .contains(&ErrorCodes::InvalidInputResponse));

            Ok(())
        }

        #[gotcha_server_macros::integration_test]
        async fn invalid_algorithm(server: TestContext) -> anyhow::Result<()> {
            let port = server.port();
            let api_secret = server.db_api_secret().await;
            let enc_key = server.db_enconding_key().await;

            let token = jsonwebtoken::encode(
                &Header::new(jsonwebtoken::Algorithm::HS512), // wrong algorithm
                &Claims::new(ResponseClaims {
                    success: true,
                    authority: SocketAddr::new([127, 0, 0, 1].into(), port),
                }),
                &EncodingKey::from_base64_secret(&enc_key)?,
            )?;

            let response = HTTP_CLIENT
                .post(format!("http://localhost:{port}/api/siteverify"))
                .form(&[("secret", api_secret), ("response", token)])
                .send()
                .await?;
            assert_eq!(response.status(), StatusCode::OK);

            let verification: VerificationResponse = response.json().await?;
            assert!(!verification.success);
            assert!(verification
                .error_codes
                .expect("must have error codes")
                .contains(&ErrorCodes::InvalidInputResponse));

            Ok(())
        }

        #[gotcha_server_macros::integration_test]
        async fn invalid_base64(server: TestContext) -> anyhow::Result<()> {
            let port = server.port();
            let api_secret = server.db_api_secret().await;

            let token = "header-garbage_ç~,-º´.eyJleHAiOjE3MzAzMDIyNDYsInN1Y2Nlc3MiOnRydWV9.9VBstXEca0JEPksQbMOEXdL_MxBvjiDgLbp0JnfsXMw";

            let response = HTTP_CLIENT
                .post(format!("http://localhost:{port}/api/siteverify"))
                .form(&[("secret", api_secret.as_str()), ("response", token)])
                .send()
                .await?;
            assert_eq!(response.status(), StatusCode::OK);

            let verification: VerificationResponse = response.json().await?;
            assert!(!verification.success);
            assert!(verification
                .error_codes
                .expect("must have error codes")
                .contains(&ErrorCodes::InvalidInputResponse));

            Ok(())
        }
    }
}
