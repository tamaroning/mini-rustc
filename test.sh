#!/bin/bash
cd `dirname $0`

success_or_stop() {
  if [ "$1" = "0" ]; then
    :
  else
    echo "Test failed"
    exit 1
  fi
}

cargo build
./tests/compile.sh
success_or_stop "$?"

./tests/fail.sh
success_or_stop "$?"

./tests/execute.sh
success_or_stop "$?"
