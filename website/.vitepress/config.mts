import { defineConfig } from 'vitepress'
import {
  groupIconMdPlugin,
  groupIconVitePlugin,
} from 'vitepress-plugin-group-icons'


// https://vitepress.dev/reference/site-config
export default defineConfig({
  base: '/os1000/',
  title: "1000 行代码的操作系统",
  description: "从零开始，用 1000 行代码编写一个操作系统。",
  cleanUrls: true,
  markdown: {
    config(md) {
      md.use(groupIconMdPlugin)
    },
  },
  vite: {
    plugins: [
      groupIconVitePlugin()
    ],
  },
  lang: 'zh-CN',
  srcExclude: ['en/**', 'ja/**', 'ko/**', 'tw/**'],
  themeConfig: {
    sidebarMenuLabel: '目录',
    outlineTitle: '本页内容',
    returnToTopLabel: '回到顶部',
    skipToContentLabel: '跳到正文',
    darkModeSwitchLabel: '外观',
    darkModeSwitchTitle: '切换到深色模式',
    lightModeSwitchTitle: '切换到浅色模式',
    docFooter: { prev: '上一章', next: '下一章' },
    notFound: {
      title: '页面不存在',
      quote: '请从目录选择章节。',
      linkLabel: '返回首页',
      linkText: '返回首页',
    },
    sidebar: [{
      text: '目录',
      items: [
        { link: '/', text: '00. 简介' },
        { link: '/zh/01-setting-up-development-environment', text: '01. 入门' },
        { link: '/zh/02-assembly', text: '02. RISC-V 101' },
        { link: '/zh/03-overview', text: '03. 总览' },
        { link: '/zh/04-boot', text: '04. 引导' },
        { link: '/zh/05-hello-world', text: '05. Hello World!' },
        { link: '/zh/06-libc', text: '06. C 标准库' },
        { link: '/zh/07-kernel-panic', text: '07. 内核恐慌（Kernel Panic）' },
        { link: '/zh/08-exception', text: '08. 异常' },
        { link: '/zh/09-memory-allocation', text: '09. 内存分配' },
        { link: '/zh/10-process', text: '10. 进程' },
        { link: '/zh/11-page-table', text: '11. 页表' },
        { link: '/zh/12-application', text: '12. 应用程序' },
        { link: '/zh/13-user-mode', text: '13. 用户模式' },
        { link: '/zh/14-system-call', text: '14. 系统调用' },
        { link: '/zh/15-virtio-blk', text: '15. 磁盘 I/O' },
        { link: '/zh/16-file-system', text: '16. 文件系统' },
        { link: '/zh/17-outro', text: '17. 结语' },
      ],
    }],
    socialLinks: [
      { icon: 'github', link: 'https://github.com/ChuanyuXue/operating-system-in-1000-lines' },
    ],
  },
})
