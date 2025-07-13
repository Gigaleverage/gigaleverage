# Gigaleverage

A modern, cross-platform application for time series data analysis and sentiment analysis, built with Rust and Slint.

## Description

This project is a GUI application designed for ingesting, analyzing, and predicting time series data. It supports various data providers and leverages sentiment analysis to improve predictions. The interface is built with Slint for a modern, native experience on macOS, Windows, and Linux.

## Getting Started

### Prerequisites

- Rust and Cargo (latest stable version)

### Running the Application

1.  Clone the repository:

    ```bash
    git clone https://github.com/Gigaleverage/gigaleverage.git
    cd gigaleverage
    ```

2.  Run the application using the Makefile:
    ```bash
    make run
    ```
    Alternatively, you can run it directly with Cargo:
    ```bash
    cargo run --bin gigaleverage
    ```

## Development

This project is structured as a Cargo workspace. The main application crate is located in `gigaleverage-app/`.

- **UI Source**: `gigaleverage-app/ui/main.slint`
- **Main Logic**: `gigaleverage-app/src/main.rs`
- **Assets**: `gigaleverage-app/assets/`
