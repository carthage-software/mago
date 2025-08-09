# Continuous Integration

This section describes how to set up Mago in a CI environment.

## GitHub Actions

Mago can be installed in GitHub Actions using the `setup-mago`
action.

```yaml
name: Code Quality

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
    - name: Checkout
      uses: actions/checkout@v2
    - name: Setup Mago 
      uses: nhedger/setup-mago@v1
    - name: Run Mago 
      run: |
        mago format --dry-run
        mago lint --reporting-format=github
```