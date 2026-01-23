import asyncio
import secrets
import string
import websockets

def gen_sri(n: int = 12) -> str:
    # Lichess/Lishogi sri examples look like URL-safe-ish mixed chars.
    alphabet = string.ascii_letters + string.digits
    return "".join(secrets.choice(alphabet) for _ in range(n))

async def keepalive(ws): # type: ignore
    # ping_counter = 0 # 3s
    version_counter = 18 # 60s


    while True:
        await asyncio.sleep(3)
        try:
            if version_counter == 19:
                await ws.send('{ "t": "version_check" }') # type: ignore
                version_counter = -1

            version_counter += 1
            await ws.send('null') # type: ignore
        except Exception:
            return

async def main():
    sri = gen_sri()
    url = f"wss://socket1.lishogi.org/watch/dP8exR8A/sente/v6?sri={sri}"

    headers = {
        # Some servers care about Origin when mimicking browser traffic
        "Origin": "https://lishogi.org",
        "User-Agent": "lishogi-ws-listener/0.1",
    }

    
    print("Connecting to:", url)
    async with websockets.connect(url, additional_headers=headers) as ws:
        print("✅ connected; sri =", sri)

        ka = asyncio.create_task(keepalive(ws))

        ping_count = 0
        prev_was_ping = False

        try:
            async for msg in ws:
                if msg == '0':
                    ping_count += 1
                    print(f"🏓 ping #{ping_count}\r", end='')
                    prev_was_ping = True
                elif str(msg).startswith('{"t":"versionCheck"'):
                    continue
                else:
                    if prev_was_ping:
                        print()
                        prev_was_ping = False
                    print("📨", msg)
        finally:
            ka.cancel()

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\n👋 exiting on user request")