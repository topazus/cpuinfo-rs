rm -rf .git/
git init
git branch -m main
git add .
git commit -m "update"
git remote add origin git@github.com:topazus/learn-on-opensuse.git
git push -f origin main
