This is my first rust project, please be forgiving!

# Why ?

This project purpose is to automate Plex/Jellyfin media deletion with a nice level of customisation.

# Moonager / Lunager

Created by: Jolan 
Created time: July 14, 2023 11:23 AM
Tags: Product

# What is Moonager and Lunager?

Moonager = Front / Lunager = Back

This software use predefined rules to auto delete medias in Plex/Jellyfin with Tautulli/Radarr/Sonarr/Overseerr/Jellyseerr

Automatically delete medias after a delay of inactivity or based on the disk pressure level.

Backend is made with Rust

Frontend is made with Nuxt

# How ?

### Delay

To get the inactivity time of a media, he use Tautulli watch history for Plex and the Playback Reporting plugin for Jellyfin.

The delay is configured with environment variable. Default delay is 2 months.

### Disk pressure

To get the disk pressure level, he use Radarr or Sonarr API based on which one is configured.

For now only one disk is handle.

# Improvements

- [ ]  Associate media with a disk for improving disk pressure management
- [ ]  Ability to filter on popularity (IMDB/OMDb)