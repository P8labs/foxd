# Foxd

This is a LAN listener that can be used for many things for example:

- We can get notification if someone connects to network and identify them that mean if my car arrives home automatically it will connect to home network and I will get notified that it arrived open the garage.
- Or we can looks if any unidentified device is connected to network.
- We can also monitor IOT devices and chech thier health by looking if they are connected to wifi, etc.

# How It Works

This project for now has two parts a console and a daemon that can be hosted on home server through a docker container or directly.

### Daemon

The work of the daemon is to listen for LAN events and notify to channels. Also serves a API server that console can utilize to show it in UI. I am writing it in rust because it can than be compiled for bare metal.

### Console

This is a simple Svelte web app where admin can see all activites and manage them. we can add rules and other configs through website.
