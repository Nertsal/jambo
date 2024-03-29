# Nertsal's Twitch Bot

Connects to a twitch account using a provided OAuth token, joins a twitch channel and interacts with viewers. Main functionality is handling a queue of games submitted for review during game jams. Commands can be called through twitch chat or console (console calls are interpreted as broadcaster calls). Autocomplete works for literals in console.

## Usage

Clone this repository `git clone https://github.com/Nertsal/nertsal-bot.git` or download a release folder (all is set up in the release, apart from **login.json** and **service_key.json** ).

Create an empty **status** folder at the root of the project. Bots' statuses will be displayed in the respective file in the readable format (currently used by GameJamBot, VoteBot, TimerBot).

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

Create a **config** folder at the root of the project with a folder inside for every bot and **active_bots.json**:
```
[
    "TimerBot",
    "QuoteBot",
    "GameJamBot",
    "CustomBot",
    "VoteBot"
]
```

See how to setup bots' configs in the respective section.

Run the executable file from the release or compile and run manually. Install cargo [here](https://doc.rust-lang.org/cargo/getting-started/installation.html). Run using `cargo run` or `cargo run --release`.

## Bots

### **ChannelsBot**

The main bot, that controls other bots.

#### Commands

- `!shutdown`. Broadcaster only. Starts shutting down, chat handle will shut down only upon receiving a message (stimulate by sending a message through twitch chat).

- `!backup create [name]`. Broadcaster only. Creates a backup named **name** or default(backup) of all bots.

- `!backup load [name]`. Broadcaster only. Loads backup named **name** or default (backup). If loading fails, a backup is created at **backups/temp**.

- `!enable <bot_name>`. Moderator only. Turns **bot_name** on.

- `!disable <bot_name>`. Moderator only. Turns **bot_name** off.

- `!reset <bot_name>`. Moderator only. Resets **bot_name** (turns it off and then back on).

### **GameJamBot**

#### Config

`config/gamejam/gamejam_config.json`:
```
{
    "multiple_submissions": false,
    "queue_mode": true,
    "return_mode": "Back",
    "auto_return": false,
    "response_time_limit": null,
    "link_start": "https://ldjam.com/events/ludum-dare/",
    "allow_direct_link_submit": true,
    "raffle_default_weight": 1,
    "google_sheet_config": {
        "sheet_id": "1zmwEZo-mKHHebHbSd_yHEp8WWqZFVZxvmRZHTvAN7ek",
        "display_luck": true,
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

- `multiple_submissions`: bool. Defines, whether viewers are allowed to submit multiple games.

- `queue_mode`: bool. Defines, whether **!queue** shows one's place in the queue and queue's length.

- `return_mode`: ReturnMode. Defines, where the game will end up after !return: `Back` or `Front` of the queue.

- `auto_return`: bool. Defines, whether **!return** will be called automatically for every message.

- `response_time_limit`: Option\<u64\>. If not null, then **!next** will require confirmation from the author, that they are in chat, to play his game. If there is no response in the given time (in seconds), then the game will be skipped, and !next will be called.

- `link_start`: Option\<String\>. If not null, then **!submit** will only allow links, which start with the given string.

- `allow_direct_link_submit`: bool. If true, then posted links, which start with **link_start**, will be submitted.

- `allow_multiple_authors_submits`: bool. If true, then viewers will be able to add themselves to the authors list of a game by calling `!submit <game_link>`. Otherwise, they can be added as an author by a moderator or by the main author (the one who submitted the game the first).

- `raffle_default_weight`: usize. Determines default weight when participating in raffles for the first time.

- `google_sheet_config`: Option\<GoogleSheetConfig\>. If not null, then current queue state will be displayed in the given google sheet. **display_luck** defines, whether viewers' luck (in raffles) will be displayed for queued games. (Requires **service_key.json** file)

#### Commands

- `!submit <game_link>`. If **link_start** is given, then **!submit** checks, that **game_link** starts with **link_start**. If **allow_direct_link_submit** is true and **link_start** is given, then **game_link** will also be interpreted as !submit **game_link**. If `allow_multiple_authors_submits` is true, such game has already been submitted, and it is in the queue (or skipped), then the person will be marked as another author of the game.

- `!authors add <author_name>`. Adds another author to the caller's game. Only the main author can call this command.

- `!authors add <game_link> <author_name>`. Moderator only. Adds another author to the game.

- `!authors remove <author_name>`. Removes an author from the caller's game. Only the main author can call this command. The last author cannot be removed.

- `!authors remove <game_link> <author_name>`. Moderator only. Adds another author to the game.

- `!return`. Returns callers's game from the skipped list to the queue.

- `!next`. Broadcaster only. Moves current game to the played list, gets the next game from the queue and sets it as current. If **response_time_limit** is given, then waits for a reply from the author. If there is no response in the given time, **!skip next** is called.

- `!next <author_name>`. Broadcaster only. Moves current game to the played list, looks for the game from <author_name>, if found, sets it as current. No response required, even if **response_time_limit** is not null.

- `!cancel`. Removes callers's game from the queue or the skipped list. Only the main author (the one who submitted the game the first) can call this command.

- `!cancel <author_name>`. Moderator only. Works just like **!cancel**, but looks for <author_name>.

- `!queue` / `!list`. If **google_sheet_config** is given, then posts a link to the google sheet, else if **queue_mode** is true, then displays queue length and one's place in the queue, if present.

- `!current`. Displays current game.

- `!skip`. Broadcaster only. Moves current game to the skipped list.

- `!skip next`. Broadcaster only. Calls **!skip** and then **!next**.

- `!skip all`. Broadcaster only. Moves all games from the queue to the skipped list.

- `!unskip`. Broadcaster only. Undoes **!skip**.

- `!unskip <author_name>`. Broadcaster only. Similar to **!unskip**, but removes game from **author_name** from the skipped list, instead of the last skipped game.

- `!stop`. Moderator only. Moves current game to the played list.

- `!force`. Moderator only. If currently waiting for response from the author, cancels waiting (the game will be kept as current).

- `!close`. Moderator only. Closes the queue, disallowing new submits.

- `!open`. Moderator only. Opens the queue, allowing new submits.

- `!raffle`. Broadcaster only. Starts the raffle. Viewers can join the raffle using !join.

- `!raffle cancel`. Broadcaster only. Cancels the raffle.

- `!raffle finish`. Broadcaster only. Finishes the raffle, chooses weighted random joined viewer and if they have submitted a game, then sets it as current, increases every joined viewer's weight by 1.

- `!join`. Join the raffle.

- `!luck`. Shows one's current luck level (in raffles).

### **CustomBot**

No config required.

#### Commands

- `!command new <command_name> <command_response>`. Moderator only. Adds a new command **command_name** (example: **!game**) with a response **command_response**.

- `!command delete <command_name>`. Moderator only. Deletes a command with the name **command_name**.

- `!command edit <command_name> <command_response>`. Moderator only. Changes **command_name** response to **command_response**.

### **QuoteBot**

No config required.

#### Commands

- `!quote`. Displays a random quote.

- `!quote add <quote_name> <new_quote>`. Moderator only. Add quote **quote_name**: **new_quote**.

- `!quote delete <quote_name>`. Moderator only. Deletes quote **quote_name**.

- `!quote edit <quote_name> <new_quote>`. Moderator only. Edits quote **quote_name** to **new_quote**.

- `!quote rename <quote_name> <new_name>`. Moderator only. Renames quote **quote_name** to **new_name**

- `!quote <quote_name>`. Displays quote **quote_name**.

### **VoteBot**

Allows to do votings with no predetermined options.

No config required

#### Commands

- `!vote start`. Broadcaster only. Starts the voting.

- `!vote finish`. Broadcaster only. Finishes the voting.

- `!vote <vote>`. Sets or updates one's vote to **vote**.

### **TimerBot**

No config required

Time format: [hh]:[mm]:[ss]

Time display format example: 05:02

#### Commands

- `!timer set [time]`. Broadcaster only. Sets the timer paused with time given or 0.

- `!timer countdown [time]`. Broadcaster only. Starts the countdown and updates time if given.

- `!timer countup [time]`. Broadcaster only. Starts the countup and updates time if given.

- `!timer pause`. Broadcaster only. Pauses the timer.

- `!timer continue`. Broadcaster only. Unpauses the timer.
