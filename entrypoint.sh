#!/bin/sh

chown appuser:appgroup /data/db.json

exec su-exec appuser "$@"
