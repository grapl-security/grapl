# Grapl Container Debugger Container

Provides a container named `grapl/debugger` that contains various utilities that
can be useful for debugging issues in other running containers.

The `attach.sh` script provides the necessary Docker command to run this
debugger container with the appropriate permissions and in the shared namespaces
of a target container that is already running.

```
attach.sh ${NAME_OF_ANOTHER_CONTAINER}
```

Once inside, you can do things like `htop`, `strace -p 1`, `ping` other
containers on the network, etc., just as though you were inside the container
(which, in a very real sense, you are).

The directory you invoke `attach.sh` from will be mounted inside the debugging
container at `/from-host`, which is also your working directory, just in case
you want to bring anything along with you on your debugging journey.

To build the container, simply run `make`.
