use std::sync::LazyLock;

use reqwest::Client;

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

mod verify_site {
    use std::time::Duration;

    use gotcha_server::{
        response_token,
        routes::{
            internal::ResponseClaims,
            public::{ErrorCodes, VerificationResponse},
        },
        test_helpers,
    };
    use reqwest::StatusCode;

    use super::*;

    #[tokio::test]
    async fn sucessful_challenge() -> anyhow::Result<()> {
        let server = test_helpers::create_server().await;
        let port = server.port();
        let token = response_token::encode(ResponseClaims { success: true })?;

        let response = HTTP_CLIENT
            .post(format!("http://localhost:{port}/api/siteverify"))
            .form(&[("secret", "api_key"), ("response", &token)])
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
        let server = test_helpers::create_server().await;
        let port = server.port();
        let token = response_token::encode(ResponseClaims { success: false })?;

        let response = HTTP_CLIENT
            .post(format!("http://localhost:{port}/api/siteverify"))
            .form(&[("secret", "api_key"), ("response", &token)])
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
        let server = test_helpers::create_server().await;
        let port = server.port();
        let token = response_token::encode(ResponseClaims { success: true })?;

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
        let server = test_helpers::create_server().await;
        let port = server.port();

        let response = HTTP_CLIENT
            .post(format!("http://localhost:{port}/api/siteverify"))
            .form(&[("secret", "api_key")])
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
        let server = test_helpers::create_server().await;
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
        let server = test_helpers::create_server().await;
        let port = server.port();
        let token = response_token::encode(ResponseClaims { success: true })?;

        let response = HTTP_CLIENT
            .post(format!("http://localhost:{port}/api/siteverify"))
            .form(&[("secret", ""), ("response", &token)])
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
        let server = test_helpers::create_server().await;
        let port = server.port();
        let token = response_token::encode(ResponseClaims { success: true })?;

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
        use gotcha_server::routes::internal::Claims;
        use jsonwebtoken::{EncodingKey, Header};
        use response_token::{JWT_ALGORITHM, JWT_SECRET_KEY_B64};

        use super::*;

        #[tokio::test]
        async fn expired_signature() -> anyhow::Result<()> {
            let server = test_helpers::create_server().await;
            let port = server.port();
            let token = response_token::encode_with_timeout(
                Duration::from_secs(0),
                ResponseClaims { success: true },
            )?;
            // expired by 1 second
            tokio::time::sleep(Duration::from_secs(1)).await;

            let response = HTTP_CLIENT
                .post(format!("http://localhost:{port}/api/siteverify"))
                .form(&[("secret", "api_key"), ("response", &token)])
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
            let server = test_helpers::create_server().await;
            let port = server.port();
            let token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..eyJleHAiOjE3MzAzMDIyNDYsInN1Y2Nlc3MiOnRydWV9.9VBstXEca0JEPksQbMOEXdL_MxBvjiDgLbp0JnfsXMw";
            //                                                ^ extra dot
            let response = HTTP_CLIENT
                .post(format!("http://localhost:{port}/api/siteverify"))
                .form(&[("secret", "api_key"), ("response", token)])
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
            let server = test_helpers::create_server().await;
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
                .form(&[("secret", "api_key"), ("response", &token)])
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
            let server = test_helpers::create_server().await;
            let port = server.port();
            let token = jsonwebtoken::encode(
                &Header::new(jsonwebtoken::Algorithm::HS512), // wrong algorithm
                &Claims::new(ResponseClaims { success: true }),
                &EncodingKey::from_base64_secret(JWT_SECRET_KEY_B64)?,
            )?;

            let response = HTTP_CLIENT
                .post(format!("http://localhost:{port}/api/siteverify"))
                .form(&[("secret", "api_key"), ("response", &token)])
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
            let server = test_helpers::create_server().await;
            let port = server.port();
            let token = "header-garbage_ç~,-º´.eyJleHAiOjE3MzAzMDIyNDYsInN1Y2Nlc3MiOnRydWV9.9VBstXEca0JEPksQbMOEXdL_MxBvjiDgLbp0JnfsXMw";

            let response = HTTP_CLIENT
                .post(format!("http://localhost:{port}/api/siteverify"))
                .form(&[("secret", "api_key"), ("response", token)])
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
