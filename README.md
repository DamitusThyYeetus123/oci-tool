# OCI Tool
A tool to create and modify OCI images without root or daemons
## Why?
Tools such as buildah and umoci use persistent storage, a container daemon, and are unsuitable for locked down workflows such as those in automated build environmenets. This tool aims to create and modify OCI-compliant images in a single, atomic command without any extra files.
