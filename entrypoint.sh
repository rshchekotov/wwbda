#!/bin/sh

./bot
while [ -f "logs/.reboot" ]; do
  ./bot
done
