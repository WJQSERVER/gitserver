window.BENCHMARK_DATA = {
  "lastUpdate": 1775495814799,
  "repoUrl": "https://github.com/WJQSERVER/git-server",
  "entries": {
    "git-server Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "114663932+WJQSERVER@users.noreply.github.com",
            "name": "WJQSERVER",
            "username": "WJQSERVER"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "af052f4fa5515e25a574ad61f06721cd0b6e0958",
          "message": "Merge pull request #1 from WJQSERVER/feat/v2-fetch-streaming\n\nFeat/v2 fetch streaming",
          "timestamp": "2026-04-07T00:58:54+08:00",
          "tree_id": "55821de9d3ce0c50411bd467222da9ab6bb3d658",
          "url": "https://github.com/WJQSERVER/git-server/commit/af052f4fa5515e25a574ad61f06721cd0b6e0958"
        },
        "date": 1775495814415,
        "tool": "cargo",
        "benches": [
          {
            "name": "concurrent_clones/clones/1",
            "value": 103671654,
            "range": "± 7998816",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/2",
            "value": 113696767,
            "range": "± 13438869",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/4",
            "value": 156399225,
            "range": "± 3488929",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/8",
            "value": 252698429,
            "range": "± 9113442",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/16",
            "value": 469877189,
            "range": "± 7157708",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/32",
            "value": 931289718,
            "range": "± 7558314",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/small",
            "value": 58556223,
            "range": "± 506664",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/medium",
            "value": 102188081,
            "range": "± 9547852",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/large",
            "value": 1568423940,
            "range": "± 38606933",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/small",
            "value": 85026685,
            "range": "± 6061737",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/medium",
            "value": 121133193,
            "range": "± 10795901",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/large",
            "value": 1469885898,
            "range": "± 4056662",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/small",
            "value": 946425,
            "range": "± 16047",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/medium",
            "value": 27723806,
            "range": "± 3774584",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/large",
            "value": 1275899718,
            "range": "± 25119206",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/small",
            "value": 205555,
            "range": "± 2153",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/medium",
            "value": 266538,
            "range": "± 2745",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/large",
            "value": 348617,
            "range": "± 2578",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}