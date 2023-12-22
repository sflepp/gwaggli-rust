# gwaggli

## Vision

gwaggli should be an integrated AI assistant that assists you in your everyday life.
It listens to you, understands you and helps you to organize your life.

## Features

It should be able to do the following things:
- [ ] Listen to you
- [ ] Understand you
- [ ] Understand when you need assistance
- [ ] Provide assistance at the right time

## Goal

gwaggli uses a microphone and a speakter to interact with you. The microphone is used to listen to your conversation and uses a speech-to-text algorithm to convert the audio to text in real time.

The text chunks are then added to a context which represents all the conversation. 

As the context grows, it is constantly analyzed and processed by an LLM to find out if advice is needed and if it is the right to output advice.

When the algorithm decides to output advice, it generates a piece of text and converts it to audio using a text-to-speech algorithm.

The goal is to provide assistance at the right time and to be as unobtrusive as possible and not feel like a chat-style assistant that waits for you to ask it something.

