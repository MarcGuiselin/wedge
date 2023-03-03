# <img src="images/icon.png" align="center"> Wedge

> A Simple, Open-Source Edge Redirector

[![PayPal Donations shield](https://img.shields.io/badge/donations-paypal-blue.svg?style=flat-square)](https://PayPal.me/marcguiselin/3USD)
![Maintenance](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg?style=flat-square)
[![License](https://img.shields.io/github/license/MarcGuiselin/wedge.svg?style=flat-square)](https://github.com/MarcGuiselin/wedge/blob/master/LICENSE)
[![Issue Count](https://img.shields.io/github/issues-raw/MarcGuiselin/wedge.svg?style=flat-square)](https://github.com/MarcGuiselin/wedge/issues)

## A choice that matters

Biting down on it's control of the PC market, unopposed by the laxing hammer of the law that [once restricted this kind of behavior](https://en.wikipedia.org/wiki/United_States_v._Microsoft_Corp.), Microsoft has continued to double down on its goals for internet browsing, search and data collection dominance.

We should not be relegated to using the tools forced upon us, especially when we don't know the true extent of their privacy, data collection and censorship.

## Why Wedge?

__🪓 Break yourself free__ from Microsoft's default browser Edge! Bundled in a one-step installer, Wedge automatically configures your system to use your default browser for things that would typically open in Microsoft Edge!

- If you ask cortana a question have it open in __Firefox 🦊__!
- If you search something in the windows taskbar have it open in __Google Chrome 🐼__! 
- If you click on a link in the signin page, have it open in __Brave 🦁__ when you log in!

A spiritual successor to [da2x's EdgeDeflector](https://github.com/da2x/EdgeDeflector), Wedge intends on fixing issues my users were having installing the former. Originally Wedge was only a simple installer script for the EdgeDeflector binary, but now it is a fully-fledged windows installer that bundles its own small standalone binary and uninstaller all built in memory-safe dependency-less rust.

## Install

  1. __Download__ the latest version of Wedge from [GitHub releases](https://github.com/MarcGuiselin/wedge/releases)
  2. __Run__ the installer once to configure your system
  3. 🚀 __Try it out!__ Use windows search and type something like “*Wedge is pretty cool*” 

## Why is everything opening in Bing?

Wedge just directs links towards whatever browser you set as default. Microsoft uses their search engine for most of these links. Luckily for you, I've made browser extensions to help redirect Bing to whatever search engine you desire!

### If you have __Firefox__ install [__Foxtana Pro__ <img src="https://img.shields.io/amo/v/foxtana-pro-redirect-cortana.svg?color=007ec6&style=flat-square" align="center"> <img src="https://img.shields.io/amo/users/foxtana-pro-redirect-cortana.svg?color=4c1&style=flat-square" align="center"> <img src="https://img.shields.io/amo/rating/foxtana-pro-redirect-cortana?color=orange&style=flat-square" align="center"> ](https://addons.mozilla.org/en-US/firefox/addon/foxtana-pro-redirect-cortana/) to redirect Bing

### If you have __Google Chrome__, __Brave__, __Opera__ or any other chrome-based browser install [__Chrometana Pro__ <img src="https://img.shields.io/chrome-web-store/v/lllggmgeiphnciplalhefnbpddbadfdi.svg?color=007ec6&style=flat-square" align="center"> <img src="https://img.shields.io/chrome-web-store/d/lllggmgeiphnciplalhefnbpddbadfdi.svg?color=4c1&style=flat-square" align="center"> <img src="https://img.shields.io/chrome-web-store/rating/lllggmgeiphnciplalhefnbpddbadfdi?color=orange&style=flat-square" align="center">](https://chrome.google.com/webstore/detail/chrometana-pro-redirect-c/lllggmgeiphnciplalhefnbpddbadfdi) to redirect Bing

## How it do what it do?

Since April 28 2016, Cortana opens searches only in Microsoft Edge to discourage users from using another search engine than Bing. Wedge puts you back in control of your default browser setting. The Wedge binary acts as a proxy between your operating system and Edge, translating the proprietary `microsft-edge:` protocol into regular internet links any browser can understand. Depending on what you set your default browser to be, the link might open in Google Chrome, Firefox or Brave; you name it!

## Uninstall

Uninstalling Wedge will fully restore system defaults changed during install. 

  1. Open windows __Apps & Features__. This can be found by searching for it in the windows taskbar.
  2. Find Wedge in the list of apps, click on it and click __Uninstall__. 
  3. Wedge will disappear from the list of Apps. (If it doesn't, try again making sure you read and confirm the dialog before clicking OK)

## Building

Wedge is written in rust so you'll need to [install rust](https://www.rust-lang.org/) on your system. Wedge compiles on rust 1.39.0 [(here's why)](https://github.com/MarcGuiselin/wedge/issues/1).

Run `setup.bat` to prepare your environment for building this project.

Build and run a debug build with `run.bat`

Build a release version with `release.bat`

## Donate

🍻 If you use or enjoy my work [buy me a drink](https://www.paypal.me/marcguiselin/3USD) or show your support by leaving a nice review on my browser extensions. Both are very appreciated! 

## License and Copyright

Please see the LICENSE for an up to date copy of the copyright and license for the entire project. Don't use my logos in your project without asking me first. If you make a derivative project off of mine please let me know! I'd love to see what people make with my work!
