#!/bin/sh

REPO=$(pwd)
DOC_DIR=/tmp/jwilm_chatbot_docs

cargo doc
rm -rf $DOC_DIR
mkdir $DOC_DIR
git clone https://github.com/jwilm/chatbot $DOC_DIR
cd $DOC_DIR
git checkout -b gh-pages origin/gh-pages
git pull origin gh-pages
cp -r $REPO/target/doc/* ./
git add *
git config user.email "jdwilm@gmail.com"
git config user.name "Joe Wilm"
git commit -m "Update docs"
git push origin gh-pages
