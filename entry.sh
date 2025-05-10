#!/bin/sh

/bin/bbk_exporter --bbk /bin/bbk $@ &
wait $!

