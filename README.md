<p align="center">
  <img width="250" height="250" src="https://user-images.githubusercontent.com/16275325/231887923-efc485e4-2626-44b6-86eb-e1d9a0094d46.png">
</p>
<h1 align="center">Rustacean GPT</h1>

Welcome, fellow coding enthusiasts! 🚀🤖 I am excited to introduce you to Rustacean GPT, my humble yet ambitious project that aims to turn the fantastic ChatGPT into a helpful, autonomous software engineer! 🧠💻 My goal is to create a tool to support senior engineers, generating useful boilerplate code, and more, Rustacean GPT is all about making the coding world a little bit easier and efficient. 🎉💡 Currently in its early, experimental stages, this repository is a cozy, academic corner for anyone with a curious and passionate spirit! So join me, friends, as we embark on this delightful journey in AI-assisted software engineering! 🌟🔧

<h3 align="center">Demo</h3>

Feast your eyes on this delightful gif of Rustacean GPT successfully crafting a Fibonacci generator! 🤩🎉 Please note that this is an ideal outcome, and to be fully transparent, it took about 10-15 reruns to achieve this result. Sometimes, ChatGPT can be a bit quirky and adventurous! 🤪🎢

<p align="center">
  <img src="https://user-images.githubusercontent.com/16275325/231880719-570896d0-961e-451c-b349-60634df64d1d.gif">
</p>

<h3 align="center">How to use Rustacean GPT</h3>

1. First things first, add your OpenAI-API key to your environment, so async-openai (which Rustacean GPT relies on) can make API calls to OpenAI on your behalf. 🗝️🔐
2. Clone the repository. 📁
3. Eager to see it in action? Run `cargo run`! But if you're up for personalizing it, follow the next steps. 🎨
4. Customize `./.config/InitialSystemPrompt.txt` as you wish. This will be sent as the System message in conversation history to ChatGPT. 💬
5. Tweak `./.config/HelpText.txt` to your liking. This comes in handy when errors occur while parsing ChatGPT's response. 🆘
6. Adjust `./.config/configuration.json` as desired (Note: the interactive session isn't heavily tested). 🔧
7. In `./src/main.rs`, you'll find code to create project requirements for ChatGPT. Modify it to suit your needs (I am planning to make this a config setup for easier swapping in the future! 🌟).

Once everything is set up just right, cargo run and enjoy the ride! 🎢

Remember, it runs non-stop, so use ctrl+c to bring it to a halt when you feel it's time. ⏹️

<h3 align="center">A Few Thoughts 💭</h3>

1. Currently, I only have access to gpt-3.5-turbo, not v4. Because of this, Rustacean_GPT hasn't completed projects more complex than the Fibonacci generator. However, I did get close with a weather CLI tool! 🌦️
2. I'm still learning the ropes of Rust, so some parts of the code might be a bit messy. Don't worry, I'll be tidying it up over time! 🧹
3. I haven't yet implemented a solid solution for handling conversations that exceed the token limit. A memory system could help, but I haven't explored that yet. If you encounter an ApiError, feel free to let it run—the code will keep track of errors and delete past messages to reduce token length. 🪄

<h3 align="center">To-Do List 📋</h3>

Here is a list of my next several to-do items for this project. I'll update the progress using emoji checkboxes:

- ⬜ Giving the AI memory (at first through Pinecone)
  - ✅ Create memory module & trait.
  - ✅ Create Pinecone API interactions.
  - ✅ Give Pinecone struct memory trait.
  - ⬜ Integrate memory into application loop. 
- ⬜ Create Token Estimator
  - ✅ Find token estimator library (or implement manually).
  - ⬜ Integrate token estimation into application loop.
- ⬜ Refactoring `mainloop`.
  - ⬜ Create a struct/module just for AI actions.
  - ⬜ Extract AI actions out of mainloop.
  - ⬜ Create a struct/module for application (as whole, to get away from `mainloop` name).
  - ⬜ Recreate application loop in new module.
- ⬜ Create a interactive command line tool for creating requirements.
- ⬜ Create a TUI for the entire thing.
- ⬜ Give the AI internet access.

Feel free to contribute or share your ideas on how to improve Rustacean GPT! Your input is always welcome. 🤗
