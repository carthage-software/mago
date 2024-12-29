# Getting Started with Mago

Welcome to Mago, the ultimate toolkit for managing and improving your PHP projects. This guide will help you get started, from installation to configuring and using Mago effectively in your workflow.

## Step 1: Install Mago

To begin, install Mago by following the instructions in the [Installation Guide](getting-started/installation.md). Mago supports multiple installation methods to suit your environment.

## Step 2: Configure Mago

Once Mago is installed, navigate to your project directory:

```bash
cd /path/to/your/project
```

### Create a Configuration File

In your project directory, create a `mago.toml` file. This configuration file will allow you to customize Mago's behavior for your project.

```bash
touch mago.toml
```

### Configure Mago

Edit the `mago.toml` file to define your source paths, formatter settings, linter rules, and plugins. For detailed guidance, refer to the [Configuration Guide](getting-started/configuration.md).

Example `mago.toml`:

```toml
[source]
paths = ["src", "tests"]
includes = ["vendor"]
```

## Step 3: Start Using Mago

After setting up your configuration, you can start using Mago's powerful commands:

- CLI Commands: Learn how to use Mago's command-line interface in the [CLI Guide](getting-started/cli.md).
- Formatters: Format your code with Mago's built-in formatters. Refer to the [Formatters Guide](formatter/) for more information.
- Linters: Analyze your code for potential issues using Mago's linters. Explore the [Linters Guide](linter/) for details.

### Example Commands

```bash
# Run the linter
mago lint

# Fix linting issues
mago fix

# Run the formatter
mago format
```

By integrating Mago into your development process, you can maintain a clean, consistent, and high-quality codebase.