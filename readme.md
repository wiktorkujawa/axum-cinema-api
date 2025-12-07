# Axum Cinema API

Welcome to the Axum Cinema API documentation. This API is designed to manage cinema-related data, including movies, halls, and sessions. Built with Rust's Axum framework, it offers a fast and reliable way to integrate cinema functionalities into your applications.

## Installation

### Prerequisites

1. **Install Shuttle CLI** (if not already installed):
   ```bash
   cargo install cargo-shuttle --locked
   ```

2. **Configure Secrets**:
   - A `Secrets.toml` file has been created in the project root
   - Update it with your MongoDB connection string and application URL:
     ```toml
     MONGODB_URI = "mongodb://localhost:27017"  # or your MongoDB connection string
     APP_URL = "http://localhost:3000"          # your frontend URL for CORS
     ```

3. **Set up MongoDB**:
   - Make sure you have MongoDB running locally, or use a MongoDB Atlas connection string

### Running Locally

```bash
cargo shuttle run
```

### Deploying

```bash
cargo shuttle deploy --allow-dirty
```

For more information, see [Shuttle documentation](https://shuttle.rs/docs/getting-started/installation)