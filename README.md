# Rustacean_GPT

Hello! Welcome to the home of `Rustacean_GPT`. This is my rust inspired version of [Auto-GPT](https://github.com/Torantulino/Auto-GPT). The entire source code is written in the rust programming language. Instead of purposing to make a generally assistive AI as Auto-GPT did, I have opted for creating one that focuses on being a software engineer. Although you could easy change this by adjusting your initial prompt. Feel free to clone and mess around. A demo is displayed below and some instructions to get you started as well.

## Demo

Here is a gif of the Rustacean_GPT successfully creating a fibonacci generator. This is an ideal set of results and to be transparent, it took about 10-15 reruns to get this result. Sometimes ChatGPT goes off the rails.

![ezgif com-video-to-gif (1)](https://user-images.githubusercontent.com/16275325/231880719-570896d0-961e-451c-b349-60634df64d1d.gif)


## Instructions For Using

1) You will need to add your OpenAI-API key to your environment so that [async-openai](https://github.com/64bit/async-openai) (which rustacean_gpt relies on), is able to make API calls to OpenAI in your behalf.
2) Clone the repository.
3) If you want to just see how it works, you can do `cargo run` at this point, otherwise use the following steps to customize to your desire.
4) Edit `./.config/InitialSystemPrompt.txt` as desired. This will be sent as the `System` message in conversation history to the ChatGPT.
5) Edit `./.config/HelpText.txt` as desired. This is additional text that will be sent when there are errors parsing ChatGPT's response.
6) Edit `./.config/configuration.json` as desired. Note: I haven't tested the interactive session much - barely at all.
7) Within `./src/main.rs` you will notice some code for creating the requirements. Use this for modifying the project requirements sent to ChatGPT. (I intend to make this a configuration setup so it's easier to swap out in the future).

Once all of that is setup as you would like, `cargo run`!

Keep in mind, it runs continuously, so you will need to use `ctrl+c` to shut it down when you aren't happy with where it is going.

## Thoughts

1) I only have access to `gpt-3.5-turbo`, and not v4. As such, I haven't been able to successfully get it to complete any projects more complicated than the fibonnaci generator. I did get close with a weather CLI tool.

2) I am not a seasoned Rust programmer, so there are parts of the code that are a complete and utter mess. I intend to clean them up overtime.

3) I didn't implement a good solution for when the conversation history get's too long and surpasses the token limit. This would be mitigated by using some sort of memory - but I haven't gotten there. If you see an ApiError, you can let it run if you would like, the code will keep track of the errors and delete past messages to cut down the token length.