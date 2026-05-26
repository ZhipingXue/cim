# this md describe the physic module control app (process module, robot, substrate cache, loadport e.g.)
    1. here some infrastructure should be designed. for example the general state control interface, general process state control, general robot transfer state control, module state, alarm reporter, data publisher(interface), error handler.
    2. it should be a .net8.0 project.
    3. each app can be deploy as system service.
    4. each app should be grpc service.
    5. in order to communicate with other apps (process), the service should also be defined a proto interface.