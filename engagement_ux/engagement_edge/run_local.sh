docker build . -t grapl_engagement_edge
docker run -i -t -e IS_LOCAL="True" -p 8900:8900 grapl_engagement_edge chalice local --no-autoreload  --host=0.0.0.0 --port=8900
