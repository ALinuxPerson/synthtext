# synthtext
A (unofficial) program which wraps the [TextSynth API], an API which can generate text from an input.

# Usage

## Setup

<p align="center">
    <img src="assets/config-generate.gif" width="640" height="480" alt="Configuration generation">
</p>

First things first, you need an API key from [TextSynth]. You can get one by [signing up] on their website. After you've
verified your email, you'll get your API key (for free!).

After this, you must generate a configuration file. This way, you don't have to pass in the API key everytime.

Run this command:

```bash
$ synthtext config generate --api-key=<your-api-key>
```

Don't worry about a warning about the parent directory not existing. It's normal, and it'll be created for you!

You can also pass a model (or engine definition as we like to call it around here) to your configuration file, however 
that will be covered later.

Let's get started!

## Generating text

In order to generate text, at the very minimum, you must pass an input text or "prompt". This is the text that will be
fed into the API. You must also first choose the method of delivering the text to you.

### Now

The first method, **now**, will display the text immediately to you (after the request is completed, of course!).

```bash
$ synthtext text-completion $prompt now
```

### Stream

The second method, **stream**, will send the text in chunks as they're generated on the fly.

```bash
$ synthtext text-completion $prompt stream
```

There are many other arguments that the `text-completion` subcommand accepts. To see those, pass the `--help` flag.

## Log probabilities

I'll admit, most people *probably* don't have a use case for this. Neither do I! However, you can see what the 
[API Documentation](https://textsynth.com/documentation.html#logprob) thinks about it:

> This endpoint returns the logarithm of the probability that a `continuation` is generated after a `context`. It can be
> used to answer questions when only a few answers (such as yes/no) are possible. It can also be used to benchmark the
> models.

In order to get the... logarithm of the probability you need at a minimum 2 arguments:
  - The **context**, the text to get match against, and
  - the **continuation**, the text that you want to know the logarithm of the probability of. Only accepts non-empty 
    strings.

Run this command like so:

```bash
$ synthtext log-probabilities "$context" "$continuation"
```

## Engine definitions

These are the metadata which is passed on to the API which is used to generate text. There are three types of officially
supported engine definitions (at least by [`textsynth`]): 

  - **GPT J 6B**: This is the default engine used when no engine definition is specified in the configuration. It is known to this program as "GptJ6B". 
    According to the documentation, it is:
    > [GPT-J] is a language model with 6 billion parameters trained on [the Pile] (825 GB of text data) published by
    > [EleutherAI]. Its main language is English but it is also fluent in several other languages. It is also trained on
    > several computer languages.
  - **Boris 6B**: It is known to the program as "Boris6B". According to the documentation, it is:
    > [Boris] is a fine tuned version of GPT-J for the French language. Use this model is you want the best performance
    > with the French language.
  - **Fairseq GPT 13B**: It is known to the program as "FairseqGPT13B". Do note that this engine definition doesn't
    always work. According to the documentation, it is:
    > [Fairseq GPT 13B] is the largest publically available English model with 13 billion parameters. Its training
    > corpus is less diverse than GPT-J but it has better performance at least on pure English language tasks. *Note:
    > support of this model is still experimental. It may stop working without notice.*
    
In order to use these engine definitions, you must find the configuration file and edit it to include the engine 
definition. To find it, run this command:

```bash
$ synthtext config find-path
```

The key is "engine_definitions". Edit it like so. For example, if you want to set the engine definitions to Boris 6B, 
edit the configuration like this:

```json
{
  "api_key": "<REDACTED>",
  "engine_definitions": "Boris6B"
}
```

# Library
The underlying library that this project uses is the [`textsynth`] library.

# License
This project is licensed under the [MIT license].

[MIT License]: LICENSE
[`textsynth`]: https://crates.io/textsynth
[TextSynth API]: https://textsynth.com
[TextSynth]: https://textsynth.com
[signing up]: https://textsynth.com/signup.html
[GPT-J]: https://github.com/kingoflolz/mesh-transformer-jax/#gpt-j-6b
[the Pile]: https://pile.eleuther.ai/
[EleutherAI]: https://www.eleuther.ai/
[Boris]: https://github.com/coteries/cedille-ai
[Fairseq GPT 13B]: https://github.com/pytorch/fairseq/tree/main/examples/moe_lm