# Actix + HTMX Project

## Introduction

Welcome to our Actix-Web and HTMX powered Rust project! This repository is an example of how Rust, along with Actix-Web and HTMX, can be used to create dynamic and efficient web applications. This project is structured to provide a clear separation of concerns, making it easier to manage and extend.

## Features

- **Actix-Web Framework**: Utilizes Actix-Web, a powerful, pragmatic, and extremely fast web framework for Rust.
- **HTMX Integration**: Leverages HTMX for enriching web pages with AJAX capabilities without writing JavaScript.
- **Modular Design**: The project is divided into modules such as `actors`, `configs`, `handlers`, and `models` for better organization.
- **Template Rendering**: Uses Tera, a template engine for Rust, for rendering HTML templates.
- **Database Integration**: Integrated with PostgREST for database interactions.
- **Sanity.io Integration**: Configured to work with Sanity.io for content management.
- **Environment Variables**: Utilizes dotenv for managing environment variables.
- **Real-Time WebSockets**: Incorporates WebSocket for real-time bidirectional communication.
- **Session Management**: Implements login/logout functionalities and cookie management.

## Structure

- `actors`: Contains actor definitions for the Actix framework.
- `configs`: Configuration files and environment setup.
- `handlers`: Request handlers for various endpoints.
- `models`: Data models and business logic.

## Endpoints

- `/`: The home page.
- `/about`: Information about the project.
- `/content`: Display content from Sanity.io.
- `/login` and `/logout`: Session management.
- `/events`: Real-time event updates via WebSockets.
- `/cookie`: Cookie handling demonstration.
- `/get_leaderboard`, `/get_comp`, `/get_content`: Examples of data retrieval.

## Getting Started

1. **Set Up Rust Environment**: Ensure you have Rust and Cargo installed.
2. **Clone the Repository**: `git clone [repo-link]`.
3. **Install Dependencies**: Run `cargo build` to install the necessary dependencies.
4. **Environment Variables**: Set up the required environment variables (e.g., `SUPABASE_URL`, `SANITY_TOKEN_KEY`).
5. **Run the Application**: Execute `cargo run` to start the server.

## Dependencies

- `actix-web`: For creating the web server and handling HTTP requests.
- `tera`: For template rendering.
- `dotenv`: To load environment variables.
- `postgrest`: For database interactions.
- `sanity`: For Sanity.io content management.

## Contribution

Contributions to this project are welcome! Please fork the repository, make your changes, and submit a pull request.

## License

This project is licensed under the MIT License. Please see the LICENSE file for more details.

---

This README provides a basic overview of the project. For more detailed information, please refer to the source code and comments within each module.
