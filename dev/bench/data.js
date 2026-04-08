window.BENCHMARK_DATA = {
  "lastUpdate": 1775677269332,
  "repoUrl": "https://github.com/WJQSERVER/gitserver",
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
      },
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
          "id": "4e9112971036b08122bdda8adf74ac9ae2ecf03c",
          "message": "Merge pull request #4 from WJQSERVER/fix/followup-cleanups\n\nFix/followup cleanups",
          "timestamp": "2026-04-07T03:09:07+08:00",
          "tree_id": "a446a62aa0af29c65d7ef65031fc41085824c70b",
          "url": "https://github.com/WJQSERVER/git-server/commit/4e9112971036b08122bdda8adf74ac9ae2ecf03c"
        },
        "date": 1775503508713,
        "tool": "cargo",
        "benches": [
          {
            "name": "concurrent_clones/clones/1",
            "value": 102553742,
            "range": "± 10281381",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/2",
            "value": 113806622,
            "range": "± 6877141",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/4",
            "value": 158502471,
            "range": "± 4784173",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/8",
            "value": 255216125,
            "range": "± 8098928",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/16",
            "value": 471612722,
            "range": "± 4784204",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/32",
            "value": 936788788,
            "range": "± 21971943",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/small",
            "value": 58569926,
            "range": "± 1172530",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/medium",
            "value": 103085262,
            "range": "± 6863561",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/large",
            "value": 1547576370,
            "range": "± 26469722",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/small",
            "value": 85301529,
            "range": "± 10951177",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/medium",
            "value": 120829015,
            "range": "± 7887664",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/large",
            "value": 1466902747,
            "range": "± 4175312",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/small",
            "value": 916315,
            "range": "± 17189",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/medium",
            "value": 29250238,
            "range": "± 3044803",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/large",
            "value": 1271766968,
            "range": "± 18687752",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/small",
            "value": 204475,
            "range": "± 1429",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/medium",
            "value": 266082,
            "range": "± 2340",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/large",
            "value": 352489,
            "range": "± 2522",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "9460fbeec82cd8c550063c43c13497f941c402f0",
          "message": "Merge pull request #7 from WJQSERVER/dependabot/cargo/rustls-webpki-0.103.10\n\nbuild(deps): bump rustls-webpki from 0.103.9 to 0.103.10",
          "timestamp": "2026-04-08T05:30:20+08:00",
          "tree_id": "7789052f75c80dc1e09928c35857dbe1d36a2307",
          "url": "https://github.com/WJQSERVER/gitserver/commit/9460fbeec82cd8c550063c43c13497f941c402f0"
        },
        "date": 1775598865406,
        "tool": "cargo",
        "benches": [
          {
            "name": "concurrent_clones/clones/1",
            "value": 113369244,
            "range": "± 8234688",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/2",
            "value": 154891307,
            "range": "± 40925421",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/4",
            "value": 251654473,
            "range": "± 9062780",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/8",
            "value": 455761740,
            "range": "± 7786296",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/16",
            "value": 892775107,
            "range": "± 15636710",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/32",
            "value": 1808527257,
            "range": "± 25080703",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/small",
            "value": 60430190,
            "range": "± 1051551",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/medium",
            "value": 112929969,
            "range": "± 8079311",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/large",
            "value": 1796621381,
            "range": "± 23249525",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/small",
            "value": 85985505,
            "range": "± 1215472",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/medium",
            "value": 125594630,
            "range": "± 13299740",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/large",
            "value": 1481277861,
            "range": "± 14962751",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/small",
            "value": 1099649,
            "range": "± 25241",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/medium",
            "value": 36462999,
            "range": "± 1446475",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/large",
            "value": 1354406821,
            "range": "± 7792131",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/small",
            "value": 207981,
            "range": "± 3250",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/medium",
            "value": 270732,
            "range": "± 12013",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/large",
            "value": 355868,
            "range": "± 3640",
            "unit": "ns/iter"
          }
        ]
      }
    ],
    "gitserver Benchmarks": [
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
          "id": "fb05600cef4201e799e833465c7403f1ed4d4830",
          "message": "Merge pull request #9 from WJQSERVER/docs\n\nDocs",
          "timestamp": "2026-04-09T03:19:07+08:00",
          "tree_id": "08eb7af359ffc4f8f2908b69d2deb8a803d82c83",
          "url": "https://github.com/WJQSERVER/gitserver/commit/fb05600cef4201e799e833465c7403f1ed4d4830"
        },
        "date": 1775677268310,
        "tool": "cargo",
        "benches": [
          {
            "name": "concurrent_clones/clones/1",
            "value": 98954022,
            "range": "± 2027564",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/2",
            "value": 135090705,
            "range": "± 53864942",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/4",
            "value": 221335545,
            "range": "± 6651404",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/8",
            "value": 387901692,
            "range": "± 6307459",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/16",
            "value": 772316297,
            "range": "± 7041288",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/32",
            "value": 1546975452,
            "range": "± 10900336",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/small",
            "value": 54857911,
            "range": "± 441041",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/medium",
            "value": 98305982,
            "range": "± 4082734",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/large",
            "value": 1639947815,
            "range": "± 29989835",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/small",
            "value": 84292180,
            "range": "± 20146238",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/medium",
            "value": 116182858,
            "range": "± 10739969",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/large",
            "value": 1275785126,
            "range": "± 16119505",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/small",
            "value": 753643,
            "range": "± 15344",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/medium",
            "value": 28866000,
            "range": "± 2456261",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/large",
            "value": 1152486373,
            "range": "± 12441165",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/small",
            "value": 118617,
            "range": "± 1298",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/medium",
            "value": 159954,
            "range": "± 12521",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/large",
            "value": 207516,
            "range": "± 3548",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}