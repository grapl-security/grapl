#######################################

FROM cypress/included:5.6.0 AS grapl-cypress

WORKDIR /test

COPY . .

RUN apt-get update && apt-get -y install --no-install-recommends wait-for-it

ENTRYPOINT [""]
CMD cypress run --browser chrome --headless
