import json

hosts = {}

# with open("./uucp.map.jsonl") as inf, open("./uucp.map.next.jsonl", "w") as out:
#     for line in inf:
#         data = json.loads(line)

#         final = {**data}

#         if (admins := final["admins"]) is not None:
#             final["admins"] = list(filter(None, admins))

#         if (peers := final["peers"]) is not None and isinstance(peers, str):
#             final["peers"] = peers.split(",")

#         if not final["os"]:
#             final["os"] = None

        # out.write(f"{json.dumps(final)}\n")

with open("./uucp.map.jsonl") as inf:
    for line in inf:
        data = json.loads(line)

        final = {**data}

        if (admins := final["admins"]) is not None:
            final["admins"] = list(filter(None, admins))

        if (peers := final["peers"]) is not None and isinstance(peers, str):
            final["peers"] = peers.split(",")

        if not final["os"]:
            final["os"] = None

        hosts[data["name"]] = data

# for name, body in hosts.items():
#     if (peers := body["peers"]) is not None:
#         for peer in peers[:]:
#             if peer not in hosts:
#                 peers.remove(peer)

for name, body in hosts.items():
    if (peers := body["peers"]) is not None:
        for peer in peers[:]:
            assert peer in hosts
