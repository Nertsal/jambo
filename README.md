# Nertsal's Twitch Bot

Connects to a twitch account using a provided OAuth token and joins a twitch channel

## Usage

Clone this repository `git clone https://github.com/Nertsal/nertsal-bot.git`

Create a **secrets** folder at the root of the project, containing 2 files:
1. `secrets/login.json`:
```
{
    "login_name": <name of the twitch account>,
    "oauth_token": <token to access account (without 'oauth:')>,
    "channel_login": <channel name to join>
}
```
2. `secrets/service_key.json` (Optional, used to access Google Sheets)

Create a **config** folder at the root of the project with a folder inside for every bot and **bots_config.json**:
```
{
    "gamejam": true,
    "custom": true,
    "quote": true,
    "reply": true
}
```

Check how to setup bots' configs at their own section.

Run using `cargo run` or `cargo run --release`.

## Bots

### **ChannelsBot**

The main bot, that controls other bots.

#### Commands

- `!enable <bot_name>`. Moderator only. Turns <bot_name> on.

- `!disable <bot_name>`. Moderator only. Turns <bot_name> off.

- `!reset <bot_name>`. Moderator only. Resets <bot_name> (turns it off and then back on).

### **GameJamBot**

#### Config

`config/gamejam/gamejam_config.json`:
```
{
    "queue_mode": true,
    "return_mode": "Back",
    "auto_return": false,
    "response_time_limit": null,
    "link_start": "https://ldjam.com/events/ludum-dare/",
    "allow_direct_link_submit": true,
    "raffle_default_weight": 1,
    "google_sheet_config": {
        "sheet_id": "1zmwEZo-mKHHebHbSd_yHEp8WWqZFVZxvmRZHTvAN7ek",
        "cell_format": {
            "color_queued": null,
            "color_current": {
                "red": 0.26,
                "green": 0.52,
                "blue": 0.96
            },
            "color_skipped": {
                "red": 1.0,
                "green": 0.0,
                "blue": 0.0
            },
            "color_played": {
                "red": 0.0,
                "green": 1.0,
                "blue": 0.0
            }
        }
    }
}
```

- `queue_mode`: bool. Defines, whether !queue command shows one's place in the queue, and its length.

- `return_mode`: ReturnMode. Defines, where the game will end up after !return: `Back` or `Front` of the queue.

- `auto_return`: bool. Defines, whether !return will be called automatically for every message.

- `response_time_limit`: Option\<u64\>. If not null, then !next will require confirmation from author, that he is in chat, to play his game. If there is no response in given time (in seconds), then the game will skipped, and !next will be called.

- `link_start`: Option\<String\>. If not null, then !submit will only allow link, which start with the given string.

- `allow_direct_link_submit`: bool. If true, then posted links, which start with **link_start**, will be submitted.

- `raffle_default_weight`: usize. Determines default weight when participating in raffles for the first time.

- `google_sheet_config`: Option\<GoogleSheetConfig\>. If not null, then current queue state will displayed in the given google sheet. (Requires **service_key.json** file)

#### Commands

- `!submit <game_link>`. If **link_start** is given, then checks, that **game_link** starts with **link_start**. If **allow_direct_link_submit** is true, then <game_link> will also be interpreted as !submit <game_link>, if **link_start** is also given.

- `!next`. Broadcaster only. Moves current game to played list, gets the next game from the queue and puts as current. If **response_time_limit** is given, then waits for a reply from author. If there is not response in given time, `!skip next` is called.

- `!next <author_name>`. Broadcaster only. Moves current game to played list, looks for the game from <author_name>, if found, sets it as current. No response ever required.

- `!cancel`. Removes one's game from the queue.

- `!cancel <author_name>`. Moderator only. Works just like **!cancel**, but looks for <author_name>.

- `!queue`. If **google_sheet_config** is given, then posts a link to the google sheet, else if **queue_mode** is true, then displays queue length, one's place in the queue, if present.

- `!current`. Displays current game.

- `!skip`. Broadcaster only. Moves current game to skipped list.

- `!skip next`. Broadcaster only. Calls **!skip** and then **!next**.

- `!skip all`. Broadcaster only. Moves all games from the queue to the skipped list.

- `!unskip`. Broadcaster only. Undoes **!skip**.

- `!stop`. Moderator only. Moves current game to played list.

- `!force`. Moderator only. If currently waiting for response from author, cancels waiting.

- `!close`. Moderator only. Closes queue, disabling new submits.

- `!open`. Moderator only. Opens queue, enabling new submits.

- `!raffle`. Broadcaster only. Starts the raffle.

- `!raffle cancel`. Broadcaster only. Cancels the raffle.

- `!raffle finish`. Broadcaster only. Finishes the raffle, chooses weighted random joined viewer, increases every joined viewer's weight by 1.

- `!join`. Join the raffle.

### **CustomBot**

No config required.

#### Commands

- `!command new <command_name> <command_response>`. Moderator only. Adds a new command <command_name> (example: **!game**) with a response <command_response>.

- `!command delete <command_name>`. Moderator only. Deletes a command with the name <command_name>.

- `!command edit <command_name> <command_response>`. Moderator only. Changes <command_name> response to <command_response>.

### **QuoteBot**

No config required.

#### Commands

- `!quote`. Displays a random quote.

- `!quote add <quote_name> <quote>`. Moderator only. Add quote <quote_name>: <quote>.

- `!quote delete <quote_name>`. Moderator only. Deletes quote <quote_name>.

- `!quote edit <quote_name> <quote>`. Moderator only. Edits quote <quote_name> to <quote>.

- `!quote rename <quote_name> <new_name>`. Moderator only. Renames quote <quote_name> to <quote>

- `!quote <quote_name>`. Displays quote <quote_name>.

### **ReplyBot**

Not recommended, doesn't work as intended.

#### Config

`config/reply/reply_config.json`:
```
{
    "responses": [
        {
            "keywords": [
                [
                    "what",
                    "how"
                ],
                [
                    "is",
                    "old"
                ],
                [
                    "your",
                    "are"
                ],
                [
                    "age",
                    "you"
                ]
            ],
            "response": "The streamer is 20 years old"
        }
    ]
}
```
