window.BENCHMARK_DATA = {
  "lastUpdate": 1775681096532,
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
          "id": "566ef2310f6cc1939eb8876a8b3efcf1817d6ab8",
          "message": "fix: use CARGO_PKG_VERSION for agent version (#10)\n\n* fix: use CARGO_PKG_VERSION for agent version in capabilities\n\n* fix: revert version to 0.0.1\n\n* chore: update Cargo.lock for version 0.0.1\n\n* style: format code with rustfmt",
          "timestamp": "2026-04-09T04:22:18+08:00",
          "tree_id": "9f13a7f803a7167d5ff4296512f5363788180c2b",
          "url": "https://github.com/WJQSERVER/gitserver/commit/566ef2310f6cc1939eb8876a8b3efcf1817d6ab8"
        },
        "date": 1775680973666,
        "tool": "cargo",
        "benches": [
          {
            "name": "concurrent_clones/clones/1",
            "value": 113444125,
            "range": "± 7034199",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/2",
            "value": 154970552,
            "range": "± 25070910",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/4",
            "value": 252101061,
            "range": "± 9671489",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/8",
            "value": 455106545,
            "range": "± 10808171",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/16",
            "value": 902852425,
            "range": "± 16927338",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/32",
            "value": 1816980275,
            "range": "± 29108938",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/small",
            "value": 61337325,
            "range": "± 1853716",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/medium",
            "value": 113414142,
            "range": "± 6413066",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/large",
            "value": 1798357236,
            "range": "± 31198236",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/small",
            "value": 86997874,
            "range": "± 4280340",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/medium",
            "value": 126012590,
            "range": "± 16452114",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/large",
            "value": 1485018983,
            "range": "± 9865821",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/small",
            "value": 1118191,
            "range": "± 46305",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/medium",
            "value": 36544833,
            "range": "± 1523210",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/large",
            "value": 1356663018,
            "range": "± 7456256",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/small",
            "value": 207852,
            "range": "± 4771",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/medium",
            "value": 270394,
            "range": "± 20189",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/large",
            "value": 357662,
            "range": "± 27223",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "114663932+WJQSERVER@users.noreply.github.com",
            "name": "wjqserver",
            "username": "WJQSERVER"
          },
          "committer": {
            "email": "114663932+WJQSERVER@users.noreply.github.com",
            "name": "wjqserver",
            "username": "WJQSERVER"
          },
          "distinct": true,
          "id": "601132e4bfce33c8431ae09db8b533a703c0e9ba",
          "message": "rename: LICENSE-UPSTREAM-MIT to UPSTREAM-LICENSE",
          "timestamp": "2026-04-09T04:23:53+08:00",
          "tree_id": "06ec8d52dd61e0d5c2ea03065a976e3cc5c364d1",
          "url": "https://github.com/WJQSERVER/gitserver/commit/601132e4bfce33c8431ae09db8b533a703c0e9ba"
        },
        "date": 1775681095972,
        "tool": "cargo",
        "benches": [
          {
            "name": "concurrent_clones/clones/1",
            "value": 115881757,
            "range": "± 5056791",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/2",
            "value": 157441535,
            "range": "± 31278130",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/4",
            "value": 255525890,
            "range": "± 7284559",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/8",
            "value": 466648218,
            "range": "± 10745072",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/16",
            "value": 919352097,
            "range": "± 14061871",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_clones/clones/32",
            "value": 1843995647,
            "range": "± 22447905",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/small",
            "value": 60257208,
            "range": "± 2825756",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/medium",
            "value": 113619389,
            "range": "± 11369593",
            "unit": "ns/iter"
          },
          {
            "name": "git_clone/clone/large",
            "value": 1879025035,
            "range": "± 19859813",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/small",
            "value": 82977022,
            "range": "± 14556011",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/medium",
            "value": 124846626,
            "range": "± 12506515",
            "unit": "ns/iter"
          },
          {
            "name": "http_clone/clone/large",
            "value": 1571225990,
            "range": "± 9535575",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/small",
            "value": 1140348,
            "range": "± 32064",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/medium",
            "value": 38602596,
            "range": "± 1955506",
            "unit": "ns/iter"
          },
          {
            "name": "pack_generation/clone/large",
            "value": 1446144108,
            "range": "± 6956345",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/small",
            "value": 213406,
            "range": "± 2428",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/medium",
            "value": 275104,
            "range": "± 4611",
            "unit": "ns/iter"
          },
          {
            "name": "ref_advertisement/advertise/large",
            "value": 369452,
            "range": "± 18198",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}