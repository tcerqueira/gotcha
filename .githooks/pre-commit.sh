#!/bin/bash

echo "Running SQLx prepare check..."

cargo sqlx prepare --check --workspace -- --all-targets --all-features
SQLX_CHECK_EXIT_CODE=$?

if [ $SQLX_CHECK_EXIT_CODE -ne 0 ]; then
    echo ""
    echo "⚠️  SQLx prepare check failed!"
    echo "This might mean your .sqlx is out of date."
    echo ""

    # Ensure we're reading from the terminal
    exec < /dev/tty

    while true; do
        read -p "Do you want to commit anyway? (y/n) " yn
        case $yn in
            [Yy]* ) exit 0;;  # Allow the commit
            [Nn]* ) exit 1;;  # Block the commit
            * ) echo "Please answer y or n.";;
        esac
    done
fi

exit 0
