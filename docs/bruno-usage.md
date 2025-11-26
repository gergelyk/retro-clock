# Bruno Usage

Bruno API client can be used for interacting with the simulator, ESP target and external services.

- Bruno files can be found in `./bruno` directory.
- In Preferences import root CA cert from `$(mkcert -CAROOT)/rootCA.pem`.
- Remember to select relevant environment: esp/sim/external.
- Remember to call *login* endpoint before calling the other endpoints that require authentication.
