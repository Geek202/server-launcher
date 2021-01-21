# server-launcher
## How to set up
1. Download the executable for your os, or build from source if there isn't one.
2. Edit your server startup script to wrap the launch in `./server-launcher`. eg.
```diff
- java -Xmx4G -Xms4G -jar fabric-server-launch.jar nogui
+ WEBHOOK_URL="<discord webhook link>" ./server-launcher java -Xmx4G -Xms4G -jar fabric-server-launch.jar nogui
```
3. The server will then auto-restart if it exits with non-zero code (crash) or the file `.restart_reason` is present (created by the `/restart` command in [server-restarter](https://github.com/Geek202/server-restarter))
