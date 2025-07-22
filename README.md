# bitcoinmints-retyr

NIP-87 Nostr event collector for ecash mint discoverability.

## Features

- Listens for NIP-87 events (38172 cashu mints, 38173 fedimints, 38000 recommendations)
- Fetches detailed mint information from `/v1/info` endpoints
- Provides REST API and web frontend for browsing mints and recommendations
- Normalizes mint URLs and handles duplicates
- Stores user profiles and recommendation data

## Logging Configuration

The application uses structured logging with proper filtering to reduce noise and improve readability.

### Log Levels

- **Application logs** (`bitcoinmints_retyr`): `INFO` level and above
- **Dependencies**: `WARN` level and above (filters out verbose debug logs from SQLx, HTTP clients, etc.)
- **Nostr SDK**: `INFO` level (important connection/subscription info)

### Environment Variables

- `RUST_LOG`: Override default log filtering (standard Rust logging env var)
- `LOG_FORMAT=json`: Use JSON formatting for production environments (default: pretty formatting)

### Examples

```bash
# Development - pretty formatted logs with emojis
cargo run

# Production - JSON formatted logs
LOG_FORMAT=json cargo run

# Debug mode - show all logs including dependencies
RUST_LOG=debug cargo run

# Custom filtering
RUST_LOG="bitcoinmints_retyr=debug,sqlx=info" cargo run
```

### Log Structure

Logs include structured fields for easy parsing:
- `target`: Identifies the component (e.g., `bitcoinmints_retyr::nostr`)
- Contextual fields: `mint_url`, `event_id`, `error`, etc.
- Emoji prefixes for quick visual scanning (🚀 startup, ✅ success, ❌ error, ⚠️ warning)

## API Endpoints

**Note:** All examples below use real data from the running database to show actual API response structures.

### `GET /`
Returns API information and available endpoints.

### `GET /api/health`
Health check endpoint. Returns status information about the service.
```json
{
  "status": "ok",
  "service": "bitcoinmints-retyr",
  "message": "NIP-87 Nostr event collector is running"
}
```



### `GET /api/mints`

Returns all discovered mints with their recommendations:
```json
{
  "mints": [
    {
      "mint": {
        "event_id": "2c48149fa176b158143ca651c1f66c904d106eee146d4320736992b83b2ededd",
        "name": "21Mint",
        "mint_url": "https://21mint.me",
        "description": "Secure and privacy-oriented Cashu mint. All logs are automatically deleted every 24 hours.",
        "mint_pubkey": "034c881b4cdb63e39ac18d1efe11c36d8c9b2ed9e0d62702d725a7a1786a3028bd",
        "author_pubkey": "8c382ee1f0df71548ca203063c0def8da4a84c6add3b43caa8469d1e3732cce7",
        "mint_type": "cashu",
        "networks": ["mainnet"],
        "invite_codes": ["https://21mint.me"],
        "nuts": ["4", "5", "7", "8", "9", "10", "11", "12", "14", "15", "17", "20"],
        "modules": [],
        "created_at": 1753113274,
        "received_at": "2025-01-21T09:47:54.000Z"
      },
      "recommendations": [
        {
          "recommendation": {
            "event_id": "890da5336e7a3d8f5e67820ef29f86a53c83b964d1a36e9e72e0ab587243ee18",
            "reviewer_pubkey": "8c382ee1f0df71548ca203063c0def8da4a84c6add3b43caa8469d1e3732cce7",
            "mint_pubkey": "034c881b4cdb63e39ac18d1efe11c36d8c9b2ed9e0d62702d725a7a1786a3028bd",
            "rating": 5,
            "content": "Follow us on Nostr for the latest news and updates! 🥜⚡️",
            "d_tag": "034c881b4cdb63e39ac18d1efe11c36d8c9b2ed9e0d62702d725a7a1786a3028bd",
            "k_tag": "38172",
            "invite_codes": ["https://21mint.me"],
            "created_at": 1753113274,
            "received_at": "2025-01-21T09:47:54.000Z"
          },
          "user_profile": {
            "pubkey": "8c382ee1f0df71548ca203063c0def8da4a84c6add3b43caa8469d1e3732cce7",
            "name": "21Mint",
            "display_name": "21Mint.me",
            "about": "21Mint By Bitcoiners for Bitcoiners.\nCashu Mint for the Plebs\n\nLeave review: https://mintpage.azzamo.net/21mint.me",
            "picture": "https://m.primal.net/NYTD.png",
            "banner": "https://m.primal.net/NYTW.jpg",
            "website": "https://bitcoinmints.com/?tab=reviews&mintUrl=https%3A%2F%2F21mint.me",
            "nip05": "_@21mint.me",
            "created_at": 1703001234,
            "received_at": "2023-12-19T15:30:45.123Z"
          }
        },
        {
          "recommendation": {
            "event_id": "89c5f6c59e1ae655ca2f66b90a4124b0483d543192072eaba63c6a98bad794b1",
            "reviewer_pubkey": "b9d5de4b4622172a12739c87d2db838148160f22d354c61ca75687a326c0a1b8",
            "mint_pubkey": "034c881b4cdb63e39ac18d1efe11c36d8c9b2ed9e0d62702d725a7a1786a3028bd",
            "rating": 5,
            "content": "works so far",
            "d_tag": "034c881b4cdb63e39ac18d1efe11c36d8c9b2ed9e0d62702d725a7a1786a3028bd",
            "k_tag": "38172",
            "invite_codes": [],
            "created_at": 1753087654,
            "received_at": "2025-01-21T02:40:54.000Z"
          },
          "user_profile": {
            "pubkey": "b9d5de4b4622172a12739c87d2db838148160f22d354c61ca75687a326c0a1b8",
            "name": "pbl",
            "display_name": "pbl",
            "about": "I am not a bot I just like breakfast\n\n",
            "picture": "https://image.nostr.build/b4e0a35d9bb4e01c266ba00c19f7df56b9620f23f1c3af5ceea8a1be3118dfed.jpg",
            "banner": null,
            "website": null,
            "nip05": "pbl4e@zaps.lol",
            "created_at": 1703000123,
            "received_at": "2023-12-19T15:08:43.456Z"
          }
        }
      ],
      "total_recommendations": 2,
      "average_rating": 5.0
    },
    {
      "mint": {
        "event_id": "54cc8a8a1738ae77d2a48d1e7d1745bac5ed9a12e5fbccfe6e5a82700db42691",
        "name": "mint.kashir.xyz",
        "mint_url": "https://mint.kashir.xyz",
        "description": null,
        "mint_pubkey": "qcelgx6c1sbt3iqf",
        "author_pubkey": "ff140c396a8f0140db85284c9cc79c6844dfd709b4155c2bd43e0009a737e4be",
        "mint_type": "cashu",
        "networks": [],
        "invite_codes": ["https://mint.kashir.xyz"],
        "nuts": ["7", "8", "9", "10", "11", "12", "14", "17", "20"],
        "modules": [],
        "created_at": 1752808842,
        "received_at": "2025-01-17T13:07:22.000Z"
      },
      "recommendations": [],
      "total_recommendations": 0,
      "average_rating": null
    }
  ]
}
```

### `GET /api/users`
Returns all users with their activity:
```json
{
  "users": [
    {
      "profile": {
        "pubkey": "8c382ee1f0df71548ca203063c0def8da4a84c6add3b43caa8469d1e3732cce7",
        "name": "21Mint",
        "display_name": "21Mint.me",
        "about": "21Mint By Bitcoiners for Bitcoiners.\nCashu Mint for the Plebs\n\nLeave review: https://mintpage.azzamo.net/21mint.me",
        "picture": "https://m.primal.net/NYTD.png",
        "banner": "https://m.primal.net/NYTW.jpg",
        "website": "https://bitcoinmints.com/?tab=reviews&mintUrl=https%3A%2F%2F21mint.me",
        "nip05": "_@21mint.me",
        "created_at": 1703001234,
        "received_at": "2023-12-19T15:30:45.123Z"
      },
      "recommendations_count": 1,
      "mints_count": 1
    },
    {
      "profile": {
        "pubkey": "b9d5de4b4622172a12739c87d2db838148160f22d354c61ca75687a326c0a1b8",
        "name": "pbl",
        "display_name": "pbl",
        "about": "I am not a bot I just like breakfast\n\n",
        "picture": "https://image.nostr.build/b4e0a35d9bb4e01c266ba00c19f7df56b9620f23f1c3af5ceea8a1be3118dfed.jpg",
        "banner": null,
        "website": null,
        "nip05": "pbl4e@zaps.lol",
        "created_at": 1703000123,
        "received_at": "2023-12-19T15:08:43.456Z"
      },
      "recommendations_count": 1,
      "mints_count": 0
    }
  ]
}
```

### `GET /api/events/raw`
Returns all raw Nostr events (for debugging):
```json
[
  {
    "id": "8f7e6d5c-4b3a-2918-7654-321098fedcba",
    "event_id": "2c48149fa176b158143ca651c1f66c904d106eee146d4320736992b83b2ededd",
    "kind": 38172,
    "pubkey": "91b14182439b1ab73ca0d5300e0495b15e4a88a1b49c3276665cdf62ef00a164",
    "content": "",
    "tags": "[[\"u\",\"https://21mint.me\",\"cashu\"],[\"nuts\",\"7,8,9,10,11,12,14,17,20\"],[\"d\",\"034c881b4cdb63e39ac18d1efe11c36d8c9b2ed9e0d62702d725a7a1786a3028bd\"]]",
    "sig": "304502210094e8c5c9a2b3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e902204b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1",
    "created_at": 1753113274,
    "received_at": "2025-01-21T09:47:54.000Z",
    "processed": false
  },
  {
    "id": "7e6d5c4b-3a29-1876-5432-109876543210",
    "event_id": "890da5336e7a3d8f5e67820ef29f86a53c83b964d1a36e9e72e0ab587243ee18",
    "kind": 38000,
    "pubkey": "8c382ee1f0df71548ca203063c0def8da4a84c6add3b43caa8469d1e3732cce7",
    "content": "[5/5] Follow us on Nostr for the latest news and updates! 🥜⚡️",
    "tags": "[[\"k\",\"38172\"],[\"u\",\"https://21mint.me\",\"cashu\"],[\"d\",\"034c881b4cdb63e39ac18d1efe11c36d8c9b2ed9e0d62702d725a7a1786a3028bd\"]]",
    "sig": "3045022100a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2022074e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4",
    "created_at": 1753113274,
    "received_at": "2025-01-21T09:47:54.000Z",
    "processed": true
  },
  {
    "id": "6d5c4b3a-2918-7654-3210-987654321098",
    "event_id": "9a8b7c6d5e4f3a2b1c0d9e8f7a6b5c4d3e2f1a0b9c8d7e6f5a4b3c2d1e0f9a8",
    "kind": 0,
    "pubkey": "8c382ee1f0df71548ca203063c0def8da4a84c6add3b43caa8469d1e3732cce7",
    "content": "{\"name\":\"21Mint\",\"about\":\"21Mint By Bitcoiners for Bitcoiners.\\nCashu Mint for the Plebs\\n\\nLeave review: https://mintpage.azzamo.net/21mint.me\",\"nip05\":\"_@21mint.me\",\"picture\":\"https://m.primal.net/NYTD.png\",\"displayName\":\"21Mint.me\",\"display_name\":\"21Mint.me\",\"website\":\"https://bitcoinmints.com/?tab=reviews&mintUrl=https%3A%2F%2F21mint.me\",\"banner\":\"https://m.primal.net/NYTW.jpg\"}",
    "tags": "[]",
    "sig": "3045022100c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2022054f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5",
    "created_at": 1703001234,
    "received_at": "2023-12-19T15:30:45.123Z",
    "processed": true
  }
]
```

### `POST /api/cleanup`
Clean up duplicate mints by normalizing URLs:
```json
{
  "status": "success",
  "message": "Mint URL normalization and duplicate cleanup completed"
}
```

### `GET /api/stats`
Get mint statistics for debugging:
```json
{
  "total_mint_events": 1020,
  "successfully_parsed": 987,
  "unique_mint_urls": 234,
  "duplicate_events": 786,
  "sample_urls": [
    "https://21mint.me",
    "https://mint.kashir.xyz",
    "https://mint.fredix.xyz",
    "https://mint.minibits.cash/Bitcoin",
    "https://cashu.rocks",
    "https://mint.coinos.io",
    "https://fedimint1qw2e3r4t5y6u7i8o9p0a1s2d3f4g5h6j7k8l9z0x1c2v3b4n5m6",
    "https://nutminer.me",
    "https://mint.nostr.fan",
    "https://mint.bitcoinplebs.com"
  ]
}
```

### `GET /api/mint-info`
Get detailed mint information from /v1/info endpoints:
```json
{
  "mint_info": [
    {
      "mint_url": "https://21mint.me",
      "last_fetched_at": "2025-01-21T11:05:55.000Z",
      "fetch_success": true,
      "error_message": null,
      "created_at": "2025-01-21T09:47:54.000Z",
      "updated_at": "2025-01-21T11:05:55.000Z",
      "mint_info": {
        "name": "21Mint",
        "pubkey": "034c881b4cdb63e39ac18d1efe11c36d8c9b2ed9e0d62702d725a7a1786a3028bd",
        "version": "Nutshell/0.17.0",
        "description": "Secure and privacy-oriented Cashu mint. All logs are automatically deleted every 24 hours.",
        "description_long": "21Mint offers a robust Chaumian Ecash service designed for seamless Bitcoin cashu and Lightning transactions. Please note: This service is in beta and is provided 'as is' without any warranties. Use at your own risk!",
        "contact": [
          {
            "method": "nostr",
            "info": "npub13suzac0smac4fr9zqvrrcr003kj2snr2m5a58j4gg6w3udejennsuc6ts3"
          }
        ],
        "motd": "Welcome to 21Mint!",
        "icon_url": "https://cdn.nostrcheck.me/8c382ee1f0df71548ca203063c0def8da4a84c6add3b43caa8469d1e3732cce7/f7168979ea9ba910daff503bdc3e7302d27a3fdbb8c90ee4a8d1abcd80ee255b.webp",
        "urls": [
          "https://21mint.me",
          "http://oxac2o2mqgcfnz6dfsc432pkpdaaskhhbghmiuut5lbylp5gmald4fad.onion"
        ],
        "time": 1753111055,
        "nuts": {
          "4": {
            "methods": [
              {
                "method": "bolt11",
                "unit": "sat",
                "min_amount": 0,
                "max_amount": 1000000,
                "description": true
              }
            ],
            "disabled": false
          },
          "5": {
            "methods": [
              {
                "method": "bolt11",
                "unit": "sat",
                "min_amount": 0,
                "max_amount": 1000000
              }
            ],
            "disabled": false
          },
          "7": {
            "supported": true
          },
          "8": {
            "supported": true
          },
          "9": {
            "supported": true
          },
          "10": {
            "supported": true
          },
          "11": {
            "supported": true
          },
          "12": {
            "supported": true
          },
          "14": {
            "supported": true
          },
          "15": {
            "methods": [
              {
                "method": "bolt11",
                "unit": "sat"
              }
            ]
          },
          "17": {
            "supported": [
              {
                "method": "bolt11",
                "unit": "sat",
                "commands": [
                  "bolt11_melt_quote",
                  "proof_state",
                  "bolt11_mint_quote"
                ]
              }
            ]
          },
          "20": {
            "supported": true
          }
        }
      }
    },
    {
      "mint_url": "https://mint.kashir.xyz",
      "last_fetched_at": "2025-01-17T14:22:33.000Z",
      "fetch_success": false,
      "error_message": "Failed to connect: Connection timeout after 30 seconds",
      "created_at": "2025-01-17T13:07:22.000Z",
      "updated_at": "2025-01-17T14:22:33.000Z"
    }
  ],
  "total_count": 2
}
```

## Architecture

- **Database**: SQLite with a single `raw_events` table storing all events as received
- **Nostr Service**: Subscribes to multiple relays and stores events
- **API Layer**: Processes raw events on-demand and returns structured data
- **Targeted User Fetching**: Only fetches user profiles for recommendation authors
- **No Complex Processing**: Events are parsed when API endpoints are called

### Data Flow

1. **Subscribe** to mint and recommendation events (38172, 38173, 38000) on multiple relays
2. **Store** raw events in SQLite database
3. **Detect recommendations** and fetch user profiles only for their authors (efficient)
4. **Parse** events on-demand when API is called  
5. **Return** structured JSON responses with linked user data

## Configuration

The server connects to these Nostr relays by default:
- wss://relay.snort.social
- wss://bitcoiner.social
- wss://relay.primal.net
- wss://relay.damus.io
- wss://relay.nostr.band
- wss://nos.lol
- wss://relay.cashumints.space

## Development

### Project Structure
```
src/
├── main.rs       # Server setup and routing
├── models.rs     # Data structures
├── database.rs   # SQLite operations
├── nostr.rs      # Nostr subscription service
└── handlers.rs   # HTTP request handlers
```

### Database Schema
```sql
CREATE TABLE raw_events (
    id TEXT PRIMARY KEY,
    event_id TEXT NOT NULL UNIQUE,
    kind INTEGER NOT NULL,
    pubkey TEXT NOT NULL,
    content TEXT NOT NULL,
    tags TEXT NOT NULL,           -- JSON serialized
    sig TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    received_at TEXT NOT NULL,
    processed BOOLEAN DEFAULT FALSE
);
```

## License

MIT License - see LICENSE file for details. 