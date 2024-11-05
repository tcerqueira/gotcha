# Gotcha

[![CI status](https://github.com/tcerqueira/gotcha/actions/workflows/general.yml/badge.svg)](https://github.com/tcerqueira/gotcha/actions/workflows/general.yml)
[![Security Audit](https://github.com/tcerqueira/gotcha/actions/workflows/audit.yml/badge.svg)](https://github.com/tcerqueira/gotcha/actions/workflows/audit.yml)
![Rust Version](https://img.shields.io/badge/rust-stable-brightgreen.svg)
![PostgreSQL](https://img.shields.io/badge/postgres-%3E%3D%2016-blue)
![TypeScript](https://img.shields.io/badge/typescript-%5E5.0.0-blue)

A reCAPTCHA-compatible implementation providing custom challenge widgets served from a Rust backend.

## Overview

Gotcha is a modular CAPTCHA system that aims to be compatible with Google's reCAPTCHA while allowing custom challenge implementations. It consists of:

- A Rust backend server that handles verification and challenge management
- A TypeScript/SolidJS-based widget API that mimics the reCAPTCHA API interface
- Sample challenge widgets (e.g. "I'm not a robot" checkbox)

## Features

- Drop-in replacement for reCAPTCHA with compatible API
- Custom challenge widget support
- PostgreSQL storage for configurations and API keys
- Docker Compose setup for development

## Getting Started

### Prerequisites

- Rust toolchain
- Node.js & npm
- Docker & Docker Compose

### Development Setup

1. Start the database:
```bash
docker-compose up -d
```

2. Run database migrations (optional):
```bash
sqlx migrate run
```

2.5. Install `cargo-watch` once:
```bash
cargo binstall cargo-watch
```

3. Start the development server:
```bash
cargo watch-server
```

4. Run the example client:
```bash
cargo watch-client
```

The server will be available at `http://localhost:8080` by default.

## Widget Integration

Add the Gotcha script to your HTML:

```html
<script src="http://localhost:8080/api.js" async defer></script>
```

Add a widget container:

```html
<div class="g-recaptcha"
     data-sitekey="YOUR_SITE_KEY">
</div>
```

## License

[Add license information]

## Authors

@tcerqueira

---

Note: This is a work in progress. Documentation and features may be incomplete.
