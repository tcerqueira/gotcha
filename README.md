# Gotcha

[![CI status](https://github.com/tcerqueira/gotcha/actions/workflows/general.yml/badge.svg)](https://github.com/tcerqueira/gotcha/actions/workflows/general.yml)
[![Security Audit](https://github.com/tcerqueira/gotcha/actions/workflows/audit.yml/badge.svg)](https://github.com/tcerqueira/gotcha/actions/workflows/audit.yml)
![Rust Version](https://img.shields.io/badge/rust-stable-brightgreen.svg)
![PostgreSQL](https://img.shields.io/badge/postgres-%3E%3D%2016-blue)
![TypeScript](https://img.shields.io/badge/typescript-%5E5.0.0-blue)

A modern, extensible CAPTCHA system built with Rust that provides a drop-in replacement for Google reCAPTCHA while supporting custom challenge widgets.

## 🎯 Overview

Gotcha is a modular CAPTCHA system designed to be fully compatible with Google's reCAPTCHA API while offering complete control over challenge implementations. It enables developers to create custom, engaging security challenges while maintaining the familiar reCAPTCHA integration pattern.

## ✨ Features

- **🔄 Drop-in Replacement**: Compatible with existing reCAPTCHA implementations
- **🎨 Custom Widgets**: Extensible challenge system with multiple widget types
- **🚀 High Performance**: Built with Rust for optimal speed and resource efficiency
- **🔐 Secure**: JWT-based verification with PostgreSQL storage

## 🏗️ Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Client Web    │    │  Gotcha Server  │    │   PostgreSQL    │
│   Application   │◄──►│   (Rust/Axum)   │◄──►│    Database     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │
         ▼                       ▼
┌─────────────────┐    ┌─────────────────┐
│   Widget API    │    │    Challenge    │
│ (TypeScript/    │    │     Widgets     │
│   SolidJS)      │    │   (Multiple)    │
└─────────────────┘    └─────────────────┘
```

### Components

- **Server**: Rust-based backend handling verification, challenge management, and API endpoints
- **Widget API**: TypeScript library providing reCAPTCHA-compatible client interface
- **Challenge Widgets**: Modular challenge implementations including:
  - `im-not-a-robot`: Classic checkbox challenge
  - `cup-stack`: Interactive cup stacking game
  - `constellation`: Star pattern recognition challenge

## 🚀 Quick Start

### Prerequisites

- **Rust** (latest stable)
- **Node.js** (v18+) & npm
- **Docker & Docker Compose**
- **cargo-make** (`cargo install cargo-make`)

### Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/tcerqueira/gotcha.git
   cd gotcha
   ```

2. **Start the database**
   ```bash
   cargo make db-up
   ```

3. **Run database migrations**
   ```bash
   cargo make db-migrate
   ```

4. **Build and start development environment**
   ```bash
   cargo make dev
   ```

5. **Build and start client playground**
   ```bash
   cargo make watch-client
   ```

The client website will be available at `http://localhost:8001`.

## 🔌 Integration

### Basic Setup

1. **Include the Gotcha script in your HTML**
   ```html
   <script src="http://localhost:8080/api.js" async defer></script>
   ```

2. **Add a widget container**
   ```html
   <div class="g-recaptcha" data-sitekey="YOUR_SITE_KEY"></div>
   ```

3. **Handle the response**
   ```javascript
   function onCaptchaSuccess(token) {
     // Send token to your server for verification
     console.log('CAPTCHA solved:', token);
   }
   ```

For more details consult Google reCPATCHA [docs](https://developers.google.com/recaptcha/intro).

### Server-Side Verification

```rust
// Example verification endpoint
async fn verify_captcha(token: &str) -> Result<bool, Error> {
    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:8080/siteverify")
        .form(&[
            ("secret", "YOUR_SECRET_KEY"),
            ("response", token),
        ])
        .send()
        .await?;

    let result: VerifyResponse = response.json().await?;
    Ok(result.success)
}
```

## 🎮 Available Widgets

### Im Not A Robot
Classic checkbox-style challenge with customizable styling and behavior.

### Cup Stack
Interactive 3D cup stacking game built with Bevy engine, compiled to WebAssembly.

### Constellation
Pattern recognition challenge where users identify constellation patterns.

## Creating Custom Widgets

1. Create a new directory in `widgets/`
2. Hookup `widget-api`
3. Add build configuration to `Makefile.toml`
4. Update documentation

<!-- ## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details. -->

---

**Note**: This project is under active development. APIs and features may change. Please check the [issues](https://github.com/tcerqueira/gotcha/issues) for known limitations and upcoming features.
