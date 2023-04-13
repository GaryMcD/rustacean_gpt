# Rustacean_GPT

Hello! Welcome to the home of `Rustacean_GPT`. This is my rust inspired version of [Auto-GPT](https://github.com/Torantulino/Auto-GPT). The entire source code is written in the rust programming language. Instead of purposing to make a generally assistive AI as Auto-GPT did, I have opted for creating one that focuses on being a software engineer. Although you could easy change this by adjusting your initial prompt. Feel free to clone and mess around. A demo is displayed below and some instructions to get you started as well.

## Demo

Here is a gif of the Rustacean_GPT successfully creating a fibonacci generator. This is an ideal set of results and to be transparent, it took about 10-15 reruns to get this result. Sometimes ChatGPT goes off the rails.

![ezgif com-video-to-gif (1)](https://user-images.githubusercontent.com/16275325/231880719-570896d0-961e-451c-b349-60634df64d1d.gif)


## Instructions For Using

1) You will need to add your OpenAI-API key to your environment so that [async-openai](https://github.com/64bit/async-openai) (which rustacean_gpt relies on), is able to make API calls to OpenAI in your behalf.
2) Clone the repository.
3) Edit `./.config/InitialSystemPrompt.txt` as desired. This will be sent as the `System` message in conversation history to the ChatGPT.
4) Edit `./.config/HelpText.txt` as desired. This is additional text that will be sent when there are errors parsing ChatGPT's response.
5) Edit `./.config/configuration.json` as desired. Note: I haven't tested the interactive session much - barely at all.
6) Within `./src/main.rs` you will notice some code for creating the requirements. Use this for modifying the project requirements sent to ChatGPT. Other than the fibonacci generator I have not been able to get a successful project completed by ChatGPT.
7) If you have ChatGPT-4 access you can change the model
