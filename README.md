# FERROX

Ferrox is a high performance & feature rich agent framework written in rust to create native hyperliquid.

## Getting started
```
cargo install ferrox
```

## Quickstart
Easily create a telegram bot agent. This will be made up of 3 agents
- Voice agent to transcribe voice message to text
- Decision agent to decide which functions to call
- Formatting agent to format the response


This will have access to
- Coingecko
- Birdseye
- Dexscreener
Send transactions to
- Hyperliquid (perps and spot trading)


The telegram bot agent will be able to read messages, images and voice messages.
```
let ferrox = Ferrox::new();
ferrox.start();
```


## Bot component.

The ferrox struct wraps around a telegram bot provided by the teloxide crate. You can override the bot with your own teloxide bot implementation if you want to.

By default, ferrox will read images, voice messages and text messages using the default bot. This feature can be useful if you want to add additional features.

```
use teloxide::Bot;
let custom_bot = Bot::from_env();
let ferrox = Ferrox::builder().with_bot(custom_bot).build();
```


## Creating an agent.
Agents are wrappers that wrap around an LLM model like gpt-4o or anthropic. They should use models which are able to call functions. In each agent, we define the functions that the agent can call. The agent can call multiple functions within itself.
Each function must implement the Action trait.

### Adding actions to an agent.

This example below will allow the agent to access datafeeds from coingecko, binance and okenx and then potentially create at echnical analysis using that data.
```
let agent = Agent::builder()
  .model("gpt-4o")
  .system_prompt("You are the world's best trading agent. You have access to datafeeds from coingecko, binance and okenx and can create technical analysis using that data. You can also create orders on hyperliquid.")
  .with_action(get_token_information_from_coingecko)
  .with_action(get_tick_information_from_binance)
  .with_action(get_tick_information_from_okex)
  .with_action(create_technical_analysis_from_tick_information)
  .with_action(create_order_on_hyperliquid)
  .build();
let bot = teloxide::Bot::from_env();
let ferrox = Ferrox::builder().with_agent(agent).with_bot(bot).build();
```

Agents can also call other agents. When a prompt is provided by the user, the outer agent can delegate the content to then call the inner agent. Infact this is how ferrox works, first there is a transcription agent that transacribes the message to text. If the user has provided text, it merely passes on the text. Next, the text is sent to the core agent which has multiple actions. Its finally called to the formatting agent.
The final agent will always output back to the telgram bot
```
let transcription_agent = 
  Agent::builder()
    .model("whisper-1")
    .system_prompt("Create concise transcriptions of the message")
    .build();

let core_agent = Agent::builder()
    .model("gpt-4o")
    .system_prompt("You are the world's best trading agent. You have access to datafeeds from coingecko, binance and okenx and can create technical analysis using that data. You can also create orders on hyperliquid.")
    .with_action(get_token_information_from_coingecko)
    .with_action(get_tick_information_from_binance)
    .with_action(get_tick_information_from_okex)
    .with_action(create_technical_analysis_from_tick_information)
    .with_action(create_order_on_hyperliquid)
    .build();

let formatting_agent = Agent::builder()
  .model("claude-3-5-sonnet")
  .system_prompt("Format the response in a way that is easy to understand")
  .build();

//We chain them in order of where we want them to be called
let chained_agents = transcription_agent.next(core_agent).next(formatting_agent);
let bot = teloxide::Bot::from_env();
let ferrox = Ferrox::builder().with_agent(chained_agents).with_bot(bot).build();
```
