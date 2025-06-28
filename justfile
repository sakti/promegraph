default:
    # cargo r -- "count(up) by (job)" -a https://demo.promlabs.com -d 60
    # cargo r -- "count(up) by (job)" -a https://demo.promlabs.com -d 1
    cargo r -- "sum(rate(prometheus_http_requests_total[1m]))" -a https://demo.promlabs.com -d 60

test:
    RUSTFLAGS="-A warnings" cargo r -- "sum(rate(prometheus_http_requests_total[1m])) by (code) > 0" -a https://demo.promlabs.com -d 5

test-local-handler:
    RUSTFLAGS="-A warnings" cargo r -q -- "sum(rate(prometheus_http_requests_total[1m])) by (handler) > 0" -a http://localhost:9090 -d 1

test-local-code:
    RUSTFLAGS="-A warnings" cargo r -q -- "sum(rate(prometheus_http_requests_total[1m])) by (code) > 0" -a http://localhost:9090 -d 1
