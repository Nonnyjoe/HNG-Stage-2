# Stage 0: Random Cat Fact Testing Guide

This guide will help you test the application endpoints and features. The server is built with Rust and Actix-web. Follow the steps below to ensure a successful test.

---

## Prerequisites

- **Rust toolchain:** Install [Rust](https://www.rust-lang.org/tools/install) if you haven't already.
- **Environment Variables:** You need to create a local `.env` file based on [`.env.example`](.env.example) (see below).

---

## Setup

1. **Clone the Repository & Navigate:**

   ```sh
   git clone https://github.com/Nonnyjoe/HNG-Stage-0
   cd HNG-Stage-0
   ```

2. **Create and Configure the Environment File:**

   - Duplicate [`.env.example`](.env.example) into a new file named `.env`.
   - Fill in the required values:
     - `URL` - URL endpoint to run your server
     - `PORT` – Port on which the server will run.
     - `EMAIL` – Email address that will be returned by the `/me` endpoint.
     - `NAME` – Full name that will be returned by the `/me` endpoint.
     - `CAT_FACT_URL` – URL for fetching cat facts (default: "https://catfact.ninja/facts").

   Example `.env` file:

   ``` javascript
      URL= 127.0.0.1
      PORT=8080
      EMAIL=test@example.com
      NAME="Test User"
      CAT_FACT_URL="https://catfact.ninja/facts"
   ```

3. **Build the Application:**

   ```sh
   cargo build
   ```


## Running the Application

Start the server by running:

```sh
cargo run
```

The server will listen on `127.0.0.1:<PORT>` as specified in the `.env` file.

---

## Testing the Endpoints

The application exposes two main endpoints:

### 1. Health Check Endpoint
- **URL:** `http://127.0.0.1:<PORT>/api/v1/healthz`
- **Method:** GET

**Expected Response:**
```json
{
  "status": "success",
  "data": {
    "health": "Server is active"
  }
}
```

**Test Using cURL:**
```sh
curl http://127.0.0.1:8080/api/v1/healthz
```
*Reference: [`check_health`](src/routes/healthz.rs)*

### 2. User Information and Cat Fact Endpoint
- **URL:** `http://127.0.0.1:<PORT>/api/v1/me`
- **Method:** GET

This endpoint returns user info (loaded from the environment config) along with a random cat fact by fetching data from an external API.

**Expected Success Response:**
```json
{
  "status": "success",
  "user": {
    "name": "Test User",
    "email": "test@example.com",
    "stack": "Rust, Actix-web, PostgreSQL"
  },
  "timestamp": "2025-10-06T12:00:00.000Z",
  "fact": "A random cat fact here"
}
```

**Test Using cURL:**

```sh
curl http://127.0.0.1:8080/api/v1/me
```

---

## Additional Testing Scenarios

- **Invalid Cat Fact API:** Disconnect from the Internet or provide an invalid `CAT_FACT_URL` in `.env` to simulate API failure. The server should return an error response with an appropriate error message.
- **Check Logs:** Review the printed logs in the terminal to verify that the cat fact URL and random page numbers are logged correctly.

---

## Troubleshooting

- **Server Not Starting:** Verify that all required environment variables are set. Check the logs for error messages.
- **Database Connection Issues:** Confirm that `DATABASE_URL` is valid and PostgreSQL is running.
- **Endpoint Failures:** Use a tool like Postman or cURL to test endpoint responses. Ensure that your firewall or network settings do not block requests.

---

## References

- [Main Application](src/main.rs)
- [Configuration](src/config/config.rs)
- [Health Check Route](src/routes/healthz.rs)
- [User and Cat Fact Route](src/routes/me.rs)

Happy testing!