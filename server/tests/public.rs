use std::sync::LazyLock;

use reqwest::Client;

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

mod verify_site {
    use std::time::Duration;

    use gotcha_server::{
        response_token,
        routes::{
            challenge::ResponseClaims,
            public::{ErrorCodes, VerificationResponse},
        },
        test_helpers::{self, DEMO_API_SECRET_B64},
    };
    use reqwest::StatusCode;

    use super::*;

    #[tokio::test]
    async fn sucessful_challenge() -> anyhow::Result<()> {
        let server = test_helpers::create_test_context().await;
        let port = server.port();
        let token = response_token::encode(
            ResponseClaims { success: true },
            test_helpers::DEMO_JWT_SECRET_KEY_B64,
        )?;

        let response = HTTP_CLIENT
            .post(format!("http://localhost:{port}/api/siteverify"))
            .form(&[("secret", DEMO_API_SECRET_B64), ("response", &token)])
            .send()
            .await?;
        assert_eq!(response.status(), StatusCode::OK);

        let verification: VerificationResponse = response.json().await?;
        assert!(verification.success);
        assert_eq!(verification.error_codes, None);

        Ok(())
    }

    #[tokio::test]
    async fn failed_challenge() -> anyhow::Result<()> {
        let server = test_helpers::create_test_context().await;
        let port = server.port();
        let token = response_token::encode(
            ResponseClaims { success: false },
            test_helpers::DEMO_JWT_SECRET_KEY_B64,
        )?;

        let response = HTTP_CLIENT
            .post(format!("http://localhost:{port}/api/siteverify"))
            .form(&[("secret", DEMO_API_SECRET_B64), ("response", &token)])
            .send()
            .await?;
        assert_eq!(response.status(), StatusCode::OK);

        let verification: VerificationResponse = response.json().await?;
        assert!(!verification.success);
        assert_eq!(verification.error_codes, None);

        Ok(())
    }

    #[tokio::test]
    async fn missing_secret() -> anyhow::Result<()> {
        let server = test_helpers::create_test_context().await;
        let port = server.port();
        let token = response_token::encode(
            ResponseClaims { success: true },
            test_helpers::DEMO_JWT_SECRET_KEY_B64,
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

    #[tokio::test]
    async fn missing_response() -> anyhow::Result<()> {
        let server = test_helpers::create_test_context().await;
        let port = server.port();

        let response = HTTP_CLIENT
            .post(format!("http://localhost:{port}/api/siteverify"))
            .form(&[("secret", DEMO_API_SECRET_B64)])
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

    #[tokio::test]
    async fn missing_secret_and_response() -> anyhow::Result<()> {
        let server = test_helpers::create_test_context().await;
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

    #[tokio::test]
    async fn invalid_secret() -> anyhow::Result<()> {
        let server = test_helpers::create_test_context().await;
        let port = server.port();
        let token = response_token::encode(
            ResponseClaims { success: true },
            test_helpers::DEMO_JWT_SECRET_KEY_B64,
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

    #[tokio::test]
    async fn bad_request() -> anyhow::Result<()> {
        let server = test_helpers::create_test_context().await;
        let port = server.port();
        let token = response_token::encode(
            ResponseClaims { success: true },
            test_helpers::DEMO_JWT_SECRET_KEY_B64,
        )?;

        let response = HTTP_CLIENT
            .post(format!("http://localhost:{port}/api/siteverify"))
            // .header("Content-Type", "application/x-www-form-urlencoded")
            .body(format!("secret=api_key&response={token}"))
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

    #[tokio::test]
    async fn duplicate() -> anyhow::Result<()> {
        // TODO
        Ok(())
    }

    mod response {
        use gotcha_server::routes::challenge::Claims;
        use jsonwebtoken::{EncodingKey, Header};
        use response_token::JWT_ALGORITHM;
        use test_helpers::DEMO_JWT_SECRET_KEY_B64;

        use super::*;

        #[tokio::test]
        async fn expired_signature() -> anyhow::Result<()> {
            let server = test_helpers::create_test_context().await;
            let port = server.port();
            let token = response_token::encode_with_timeout(
                Duration::from_secs(0),
                ResponseClaims { success: true },
                test_helpers::DEMO_JWT_SECRET_KEY_B64,
            )?;
            // expired by 1 second
            tokio::time::sleep(Duration::from_secs(1)).await;

            let response = HTTP_CLIENT
                .post(format!("http://localhost:{port}/api/siteverify"))
                .form(&[("secret", DEMO_API_SECRET_B64), ("response", &token)])
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

        #[tokio::test]
        async fn immature_signature() -> anyhow::Result<()> {
            // TODO
            Ok(())
        }

        #[tokio::test]
        async fn invalid_token() -> anyhow::Result<()> {
            let server = test_helpers::create_test_context().await;
            let port = server.port();
            let token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..eyJleHAiOjE3MzAzMDIyNDYsInN1Y2Nlc3MiOnRydWV9.9VBstXEca0JEPksQbMOEXdL_MxBvjiDgLbp0JnfsXMw";
            //                                                ^ extra dot
            let response = HTTP_CLIENT
                .post(format!("http://localhost:{port}/api/siteverify"))
                .form(&[("secret", DEMO_API_SECRET_B64), ("response", token)])
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

        #[tokio::test]
        async fn invalid_signature() -> anyhow::Result<()> {
            let server = test_helpers::create_test_context().await;
            let port = server.port();
            let token = jsonwebtoken::encode(
                &Header::new(JWT_ALGORITHM),
                &Claims::new(ResponseClaims { success: true }),
                &EncodingKey::from_base64_secret(
                    "bXktd3Jvbmctc2VjcmV0", /* `my-wrong-secret` in base64 */
                )?,
            )?;

            let response = HTTP_CLIENT
                .post(format!("http://localhost:{port}/api/siteverify"))
                .form(&[("secret", DEMO_API_SECRET_B64), ("response", &token)])
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

        #[tokio::test]
        async fn invalid_algorithm() -> anyhow::Result<()> {
            let server = test_helpers::create_test_context().await;
            let port = server.port();
            let token = jsonwebtoken::encode(
                &Header::new(jsonwebtoken::Algorithm::HS512), // wrong algorithm
                &Claims::new(ResponseClaims { success: true }),
                &EncodingKey::from_base64_secret(DEMO_JWT_SECRET_KEY_B64)?,
            )?;

            let response = HTTP_CLIENT
                .post(format!("http://localhost:{port}/api/siteverify"))
                .form(&[("secret", DEMO_API_SECRET_B64), ("response", &token)])
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

        #[tokio::test]
        async fn invalid_base64() -> anyhow::Result<()> {
            let server = test_helpers::create_test_context().await;
            let port = server.port();
            let token = "header-garbage_ç~,-º´.eyJleHAiOjE3MzAzMDIyNDYsInN1Y2Nlc3MiOnRydWV9.9VBstXEca0JEPksQbMOEXdL_MxBvjiDgLbp0JnfsXMw";

            let response = HTTP_CLIENT
                .post(format!("http://localhost:{port}/api/siteverify"))
                .form(&[("secret", DEMO_API_SECRET_B64), ("response", token)])
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
