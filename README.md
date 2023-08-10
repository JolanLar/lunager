This is my first rust project, please be forgiving!

# Moonager / Lunager

Created by: Jolan 
Created time: July 14, 2023 11:23 AM
Tags: Product

# What is Moonager and Lunager?

Moonager = Front / Lunager = Back

This is an manager for Plex with Tautulli / Jellyfin with track managed with Radarr / Sonarr / Overseerr / Jellyseerr.

He automatically delete medias after a delay of inactivity or based on the disk pressure level.

Backend is made with Rust

Frontend is made with TO DEFINE

# How did he do that ?

### Delay

For getting the inactivity delay of a media, he use Tautulli watch history for Plex and the Playback Reporting plugin for Jellyfin.

The delay is configured with environment variable. Default delay is 2 months.

### Disk pressure

For get the disk pressure level, he use Radarr or Sonarr API based on which one is configured.

For now only one disk is handle.

# Is there ressources to use already ?

Yes, all needed APIs have already been analyzed, you can find useful requests in Postman.

# Improvements

- [ ]  Associate media with a disk for improving disk pressure management