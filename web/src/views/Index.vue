<template>
  <div class="index-wrapper" :class="{ night: isNight, day: !isNight }">
    <div class="navigation-wrapper" :class="[navigationClass, isWebApp && !isNight ? 'status-bar-light-bg' : '']" :style="navigationStyle" @touchstart="handleTouchStart" @touchmove="handleTouchMove" @touchend="handleTouchEnd" v-if="$store.getters.isNormalPage">
      <div class="navigation-inner-wrapper">
        <div class="navigation-title"> 阅读 <span class="version-text" @click="updateForce">{{$store.state.version}}</span></div>
        <div class="navigation-sub-title"> 清风不识字，何故乱翻书 </div>
        <div class="search-wrapper" v-show="$store.getters.hasLogin">
          <el-input size="mini" placeholder="搜索书籍" v-model="search" class="search-input" @keyup.enter.native="searchBook(1)"><i slot="prefix" class="el-input__icon el-icon-search"></i></el-input>
        </div>
        <div class="setting-wrapper search-setting" v-show="$store.getters.hasLogin">
          <div class="setting-title"> 搜索设置 </div>
          <div class="setting-item">
            <el-select size="mini" v-model="searchConfig.searchType" class="setting-select" filterable placeholder="请选择搜索方式"><el-option v-for="(item, index) in searchTypeList" :key="'search-type-' + index" :label="item.name" :value="item.value"></el-option></el-select>
          </div>
          <div class="setting-item" v-show="searchConfig.searchType === 'single'">
            <VirtualSelect v-if="shelfConfig.virtualOptimize === 'yes'" :data-list="bookSourceList" v-model="searchConfig.bookSourceUrl" />
            <el-select v-else size="mini" v-model="searchConfig.bookSourceUrl" class="setting-select" filterable placeholder="请选择搜索书源"><el-option v-for="(item, index) in bookSourceList" :key="'source-' + index" :label="item.bookSourceName" :value="item.bookSourceUrl"></el-option></el-select>
          </div>
          <div class="setting-item" v-show="searchConfig.searchType === 'single'">
            <el-tag type="info" :effect="isNight ? 'dark' : 'light'" class="setting-btn" @click="searchBookManual"> 精确搜书 </el-tag><el-tag type="info" :effect="isNight ? 'dark' : 'light'" class="setting-btn" @click="saveBookManual"> 手动加书 </el-tag>
          </div>
          <div class="setting-item" v-show="searchConfig.searchType !== 'single'">
            <el-select size="mini" v-model="searchConfig.bookSourceGroup" class="setting-select" filterable placeholder="请选择搜索书源分组"><el-option v-for="(item, index) in bookSourceGroupList" :key="'source-group-' + index" :label="item.name + ' (' + item.count + ')'" :value="item.value"></el-option></el-select>
          </div>
          <div class="setting-item" v-show="searchConfig.searchType !== 'single'">
            <el-select size="mini" v-model="searchConfig.concurrentCount" class="setting-select" filterable placeholder="请选择并发线程"><el-option v-for="(item, index) in concurrentList" :key="'source-' + index" :label="item + '并发线程'" :value="item"></el-option></el-select>
          </div>
        </div>
        <div class="recent-wrapper" v-show="$store.getters.hasLogin">
          <div class="recent-title"> 最近阅读 <span class="right-text" @click="$store.commit('clearReadingBook')">清除</span></div>
          <div class="reading-recent">
            <el-tag type="warning" :effect="isNight ? 'dark' : 'light'" class="recent-book" @click="toDetail(readingRecent)" :class="{ 'no-point': readingRecent.bookUrl == '' }"> {{ readingRecent.name }} </el-tag>
          </div>
        </div>
        <div class="setting-wrapper" v-show="$store.getters.hasLogin">
          <div class="setting-title"> 后端设定 <span class="right-text" v-if="isTauri" @click="$router.push({ path: '/setting' })">设置</span></div>
          <div class="setting-item">
            <el-tag :type="connectType" :effect="isNight ? 'dark' : 'light'" class="setting-connect" :class="{ 'no-point': connecting }" @click="setIP"> {{ connectStatus }} </el-tag>
          </div>
        </div>
        <div class="setting-wrapper" v-show="$store.getters.hasLogin">
          <div class="setting-title"> 书源设置 </div>
          <div class="setting-item">
            <el-tag type="info" :effect="isNight ? 'dark' : 'light'" class="setting-btn" @click="showBookSourceManageDialog = true"> 书源管理 </el-tag>
            <el-popover placement="right" :width="popupWidth" trigger="click" :visible-arrow="false" v-model="popExploreVisible" popper-class="popper-component explore-popover">
              <Explore ref="popExplore" class="popup" :visible="popExploreVisible" :bookSourceList="bookSourceList" @showSearchList="showSearchList" @close="popExploreVisible = false" />
              <el-tag type="info" :effect="isNight ? 'dark' : 'light'" slot="reference" ref="exploreBtn" class="setting-btn" @click="showNavigation = false"> 探索书源 </el-tag>
            </el-popover>
            <el-tag type="info" :effect="isNight ? 'dark' : 'light'" class="setting-btn" @click="uploadBookSource"> 导入书源 </el-tag>
            <el-tag type="info" :effect="isNight ? 'dark' : 'light'" class="setting-btn" @click="showRemoteBookSourceSubDialog"> 书源订阅 </el-tag>
            <el-tag type="info" :effect="isNight ? 'dark' : 'light'" class="setting-btn" @click="showFailureBookSource()"> 失效书源 </el-tag>
            <el-tag type="info" :effect="isNight ? 'dark' : 'light'" class="setting-btn" @click="debugBookSource()"> 调试书源 </el-tag>
            <input ref="fileRef" type="file" @change="onSourceFileChange" style="display:none" />
          </div>
        </div>
        <div class="setting-wrapper" v-show="$store.getters.hasLogin">
          <div class="setting-title"> 书架设置 </div>
          <div class="setting-item">
            <el-tag type="info" :effect="isNight ? 'dark' : 'light'" class="setting-btn" @click="showShelfSettingsDialog = !showShelfSettingsDialog"> 书架设置 </el-tag>
            <el-tag type="info" :effect="$store.getters.isNight ? 'dark' : 'light'" class="setting-btn" @click="showBookManage"> 书籍管理 </el-tag>
            <el-tag type="info" :effect="$store.getters.isNight ? 'dark' : 'light'" class="setting-btn" @click="showManageBookGroup"> 分组管理 </el-tag>
            <el-tag type="info" :effect="$store.getters.isNight ? 'dark' : 'light'" class="setting-btn" @click="importLocalBook"> 导入书籍 </el-tag>
            <input ref="bookRef" type="file" multiple="multiple" @change="onBookFileChange" style="display:none" />
            <el-tag type="info" :effect="$store.getters.isNight ? 'dark' : 'light'" class="setting-btn" @click="showFileManagerDialog('__LOCAL_STORE__', '书仓文件管理')" v-if="!$store.state.isSecureMode || $store.state.userInfo.enableLocalStore"> 浏览书仓 </el-tag>
            <el-tag type="info" :effect="isNight ? 'dark' : 'light'" class="setting-btn" @click="showBookmarkDialog"> 书签管理 </el-tag>
            <el-tag type="info" :effect="isNight ? 'dark' : 'light'" class="setting-btn" @click="showReplaceRuleDialog"> 替换规则 </el-tag>
            <el-tag type="info" :effect="isNight ? 'dark' : 'light'" class="setting-btn" @click="init(true)"> 刷新缓存 </el-tag>
          </div>
        </div>
        <div class="setting-wrapper">
          <div class="setting-title"> 用户空间 <span class="right-text" v-if="$store.getters.hasLogin" @click="logout()">注销</span><span class="right-text" v-else-if="$store.state.isSecureMode" @click="$store.commit('setShowLogin', true)">登录</span></div>
          <div class="setting-item" v-if="$store.state.isManagerMode">
            <el-select size="mini" v-model="userNS" class="setting-select" filterable placeholder="请选择用户空间"><el-option v-for="(item, index) in userList" :key="'source-' + index" :label="item.username" :value="item.userNS"></el-option></el-select>
          </div>
          <div class="setting-item" v-show="$store.getters.hasLogin">
            <el-tag type="info" :effect="isNight ? 'dark' : 'light'" class="setting-btn" v-if="isShowActiveLicenseBtn" @click="showActiveLicenseDialog"> 授权管理 </el-tag>
            <el-tag type="info" :effect="isNight ? 'dark' : 'light'" class="setting-btn" @click="showLicense"> 查看授权信息 </el-tag>
            <el-tag type="info" :effect="isNight ? 'dark' : 'light'" class="setting-btn" @click="saveUserConfig" v-if="localStorageAvaliable"> 备份我的配置 </el-tag>
            <el-tag type="info" :effect="isNight ? 'dark' : 'light'" class="setting-btn" @click="restoreUserConfig" v-if="localStorageAvaliable"> 同步我的配置 </el-tag>
            <el-tag type="info" :effect="isNight ? 'dark' : 'light'" class="setting-btn" @click="showFileManagerDialog('__HOME__', '用户数据管理')"> 我的数据管理 </el-tag>
            <el-tag type="info" :effect="isNight ? 'dark' : 'light'" class="setting-btn" @click="downloadBackupFile"> 下载数据备份 </el-tag>
            <el-tag type="info" :effect="isNight ? 'dark' : 'light'" class="setting-btn" v-if="$store.state.showManagerMode && !$store.state.isManagerMode" @click="loadUserList"> 进入管理模式 </el-tag>
            <el-tag type="info" :effect="isNight ? 'dark' : 'light'" class="setting-btn" v-if="$store.state.isManagerMode" @click="showUserManageDialog()"> 用户权限管理 </el-tag>
            <el-tag type="info" :effect="isNight ? 'dark' : 'light'" class="setting-btn" v-if="$store.state.isManagerMode" @click="showFileManagerDialog('__STORAGE__', '数据目录管理')"> 数据目录管理 </el-tag>
            <el-tag type="info" :effect="isNight ? 'dark' : 'light'" class="setting-btn" v-if="$store.state.isManagerMode" @click="exitSecureMode"> 退出管理模式 </el-tag>
          </div>
        </div>
        <div class="setting-wrapper" v-if="!$store.state.isSecureMode || $store.state.userInfo.enableWebdav">
          <div class="setting-title" v-show="$store.getters.hasLogin"> WebDAV </div>
          <div class="setting-item">
            <el-tag type="info" :effect="isNight ? 'dark' : 'light'" class="setting-btn" @click="showFileManagerDialog('__WEBDAV__', 'WebDAV文件管理')"> 文件管理 </el-tag>
            <el-tag type="info" :effect="isNight ? 'dark' : 'light'" class="setting-btn" @click="backupToWebdav"> 保存备份 </el-tag>
          </div>
        </div>
        <div class="setting-wrapper">
          <div class="setting-title"> 本地缓存 <span class="right-text">{{localCacheStats.total}}</span></div>
          <div class="setting-item">
            <el-tag type="info" :effect="$store.getters.isNight ? 'dark' : 'light'" class="setting-btn" @click="clearCache('bookSourceList')"> 清空书源缓存 <span>{{localCacheStats.bookSourceList}}</span></el-tag>
            <el-tag type="info" :effect="$store.getters.isNight ? 'dark' : 'light'" class="setting-btn" @click="clearCache('rssSources')"> 清空RSS源缓存 <span>{{localCacheStats.rssSources}}</span></el-tag>
            <el-tag type="info" :effect="$store.getters.isNight ? 'dark' : 'light'" class="setting-btn" @click="clearCache('chapterList')"> 清空章节列表缓存 <span>{{localCacheStats.chapterList}}</span></el-tag>
            <el-tag type="info" :effect="$store.getters.isNight ? 'dark' : 'light'" class="setting-btn" @click="clearCache('chapterContent')"> 清空章节内容缓存 <span>{{localCacheStats.chapterContent}}</span></el-tag>
            <el-tag type="info" :effect="$store.getters.isNight ? 'dark' : 'light'" class="setting-btn" @click="clearCache('ttsData')"> 清空TTS缓存 <span>{{localCacheStats.ttsData}}</span></el-tag>
          </div>
        </div>
      </div>
      <div class="bottom-icons">
        <a href="https://github.com/hectorqin/reader" target="_blank"><div class="bottom-icon"><img v-if="isNight" :src="require('../assets/imgs/github.png')" alt="" /><img v-else :src="require('../assets/imgs/github2.png')" alt="" /></div></a>
        <span class="theme-item" :style="themeColor" ref="themes" @click="toogleNight"><i class="el-icon-moon" v-if="!isNight"></i><i class="el-icon-sunny" v-else></i></span>
      </div>
    </div>
    <div class="shelf-wrapper" :class="isWebApp && !isNight ? 'status-bar-light-bg' : ''" ref="shelfWrapper" @click="hideMenu">
      <div class="shelf-title">
        <i class="el-icon-menu" v-if="$store.getters.isNormalPage && collapseMenu" @click.stop="toggleMenu"></i> {{ isSearchResult ? isExploreResult ? "探索" : isShowSearchBookSourceListDesc ? "书源" : "搜索" : "书架" }} ({{ bookList.length }})
        <div class="title-btn" v-if="$store.getters.isNormalPage && isSearchResult" @click="backToShelf"> 书架 </div>
        <div class="title-btn" v-if="$store.getters.isNormalPage && isSearchResult && !isShowSearchBookSourceListDesc" @click="loadMore"><i class="el-icon-loading" v-if="loadingMore"></i> {{ loadingMore ? "加载中..." : "加载更多" }} </div>
        <div class="title-btn" v-if="$store.getters.isNormalPage && isSearchResult && isShowSearchBookSourceListDesc" @click="displaySearchResult"> {{ "返回搜索" }} </div>
        <div class="title-btn" v-if="$store.getters.isNormalPage && !isSearchResult" @click="showBookEditButton = !showBookEditButton"> {{ showBookEditButton ? "取消" : "编辑" }} </div>
        <div class="title-btn" v-if="!isSearchResult" @click="refreshShelf"><i class="el-icon-loading" v-if="refreshLoading"></i> {{ refreshLoading ? "刷新中..." : "刷新" }} </div>
        <div class="title-btn" v-if="$store.getters.isNormalPage && !isSearchResult" @click="showRssDialog"> RSS </div>
        <div class="title-btn" @click="showExplorePop" v-if="$store.getters.isNormalPage && !(isSearchResult && !isExploreResult)"> 书海 </div>
        <el-input v-if="showBookEditButton" v-model="keyword" placeholder="搜索书架" size="mini"></el-input>
      </div>
      <div class="book-group-wrapper" v-if="!isSearchResult">
        <el-tabs class="book-group-tabs" v-model="showBookGroupString" stretch><el-tab-pane v-for="group in bookGroupDisplayList" :label="group.groupName" :name="'' + group.groupId" :key="'bookGroup-' + group.groupId"></el-tab-pane></el-tabs>
      </div>
      <div class="books-wrapper" ref="bookList" @touchstart="handleTouchStart" @touchmove="handleTouchMove" @touchend="handleTouchEnd" @scroll="scrollHandler">
        <template v-if="(!isSearchResult && shelfConfig.viewCate !== 'list') || shelfConfig.virtualOptimize !== 'yes'">
          <BookList v-if="isSearchResult || shelfConfig.viewCate === 'list'" ref="bookListComp" :book-list="bookList" :show-navigation="showNavigation" :is-search-result="isSearchResult" :show-book-edit-button="showBookEditButton" :show-source-count="!isShowSearchBookSourceListDesc" @showBookInfoDialog="showBookInfoDialog" @toDetail="toDetail" @deleteBook="deleteBook" @editBook="editBook" @addBookToShelf="addBookToShelf" @showSearchBookSourceList="showSearchBookSourceList" />
          <BookColumn v-else ref="bookListComp" :book-list="bookList" :show-navigation="showNavigation" :is-search-result="isSearchResult" :show-book-edit-button="showBookEditButton" @showBookInfoDialog="showBookInfoDialog" @toDetail="toDetail" @deleteBook="deleteBook" @editBook="editBook" @addBookToShelf="addBookToShelf" />
        </template>
        <BookVirtualList v-else ref="bookListComp" :book-list="bookList" :show-navigation="showNavigation" :is-search-result="isSearchResult" :show-book-edit-button="showBookEditButton" :show-source-count="!isShowSearchBookSourceListDesc" @showBookInfoDialog="showBookInfoDialog" @toDetail="toDetail" @deleteBook="deleteBook" @editBook="editBook" @addBookToShelf="addBookToShelf" @showSearchBookSourceList="showSearchBookSourceList" />
      </div>
    </div>
    <el-dialog :title="isImportRssSource ? '导入RSS源' : '导入书源'" :visible.sync="showImportSourceDialog" :width="dialogWidth" :top="this.collapseMenu ? '0' : '15vh'" :fullscreen="collapseMenu" :class="isWebApp && !isNight ? 'status-bar-light-bg' : ''" v-if="$store.getters.isNormalPage">
      <div class="source-container source-list-container">
        <el-checkbox-group v-model="checkedSourceIndex" @change="handleCheckedSourcesChange"><el-checkbox v-for="(source, index) in importSourceList" :label="index" :key="index" class="source-checkbox">{{ isImportRssSource ? source.sourceName : source.bookSourceName }} {{ isImportRssSource ? source.sourceUrl : source.bookSourceUrl }} {{ getSourceTag(source) }}</el-checkbox></el-checkbox-group>
      </div>
      <div slot="footer" class="dialog-footer">
        <el-checkbox :indeterminate="isIndeterminate" v-model="checkAll" @change="handleCheckAllChange" border size="medium" class="float-left checkbox-btn">全选</el-checkbox>
        <el-checkbox border size="medium" class="float-left checkbox-btn" @change="handleCheckWebviewChange">Webview 源</el-checkbox>
        <el-checkbox border size="medium" class="float-left checkbox-btn" @change="handleCheckJSChange">JavaScript 源</el-checkbox>
        <span class="check-tip">已选择 {{ checkedSourceIndex.length }} 个</span>
        <el-button size="medium" @click="showImportSourceDialog = false; checkedSourceIndex = [];">取消</el-button>
        <el-button size="medium" type="primary" @click="saveSourceList">确定</el-button>
      </div>
    </el-dialog>
    <el-dialog :visible.sync="showBookSourceManageDialog" :width="dialogWidth" :top="dialogTop" @closed="isShowFailureBookSource = false; showSourceGroup = '全部';" :fullscreen="collapseMenu" :class="isWebApp && !isNight ? 'status-bar-light-bg-dialog' : ''" v-if="$store.getters.isNormalPage">
      <div class="custom-dialog-title" slot="title"><span class="el-dialog__title">{{ isShowFailureBookSource ? "失效书源管理" : "书源管理" }} <span v-if="!isShowFailureBookSource" class="float-right span-btn" @click="deleteAllBookSource()">清空</span><span v-if="!isShowFailureBookSource" class="float-right span-btn" @click="deleteBookSourceFile()">恢复默认</span><span v-if="!isShowFailureBookSource" class="float-right span-btn" @click="exportBookSource()">导出</span><span v-if="!isShowFailureBookSource" class="float-right span-btn" @click="editBookSource(false)">新增</span></span></div>
      <div class="source-container table-container">
        <div class="check-form" v-if="isShowFailureBookSource">
          <span class="check-form-label">搜索词：</span>
          <el-input v-model="checkBookSourceConfig.keyword" size="small"></el-input>
          <span class="check-form-label" style="min-width: 68px;"> 超时(ms)： </span>
          <el-input-number v-model="checkBookSourceConfig.timeout" :min="1000" :max="15000" :step="500" size="small"></el-input-number>
          <span class="check-form-label">并发数：</span>
          <el-input-number v-model="checkBookSourceConfig.concurrent" :min="3" :max="15" :step="1" size="small"></el-input-number>
        </div>
        <div class="source-group-wrapper">
          <el-tabs class="booksource-group-tabs" v-model="showSourceGroup" stretch><el-tab-pane v-for="name in bookSourceShowGroup" :key="'sourceGroupTab-' + name" :label="name" :name="name"></el-tab-pane></el-tabs>
        </div>
        <el-table :data="bookSourceShowResultPageList" :height="dialogContentHeight - 42 - 42 - (isShowFailureBookSource ? 32 : 0)" @selection-change="manageSourceSelection = $event" :key="isShowFailureBookSource">
          <el-table-column type="selection" width="25" :fixed="$store.state.miniInterface" :selectable="isBookSourceSelectable"></el-table-column>
          <el-table-column property="bookSourceName" label="书源名称" min-width="60" sortable :fixed="$store.state.miniInterface"></el-table-column>
          <el-table-column property="bookSourceUrl" label="书源链接" min-width="120">
            <template slot-scope="scope"><el-link type="primary" :href="scope.row.bookSourceUrl" target="_blank">{{ scope.row.bookSourceUrl }}</el-link></template>
          </el-table-column>
          <el-table-column property="errorMsg" label="错误信息" min-width="120" sortable v-if="isShowFailureBookSource"></el-table-column>
          <el-table-column label="书架书籍" min-width="120">
            <template slot-scope="scope"><pre>{{ showSourceBook(scope.row) }}</pre></template>
          </el-table-column>
          <el-table-column width="120px" v-if="!isShowFailureBookSource">
            <template slot-scope="scope"><el-button type="text" @click="editBookSource(scope.row)">编辑</el-button><el-button type="text" @click="editBookSourceHeader(scope.row)">请求头</el-button></template>
            <template slot="header"><el-input v-model="bookSourceKeyword" size="mini" placeholder="搜索书源"></el-input></template>
          </el-table-column>
        </el-table>
      </div>
      <div slot="footer" class="dialog-footer">
        <div>
          <el-button type="primary" class="float-left" size="medium" @click="deleteBookSourceList">批量删除<span v-if="manageSourceSelection.length"> ({{ manageSourceSelection.length }}) </span></el-button>
          <el-button type="primary" size="medium" v-if="isShowFailureBookSource" style="margin-bottom: 5px;" @click="checkBookSource">{{ isCheckingBookSource ? "正在" : "" }}检测书源 {{ checkBookSourceTip }}</el-button>
        </div>
        <div class="source-pagination">
          <el-pagination :current-page.sync="bookSourcePagination.page" :page-sizes="[25, 50, 100, 200, 300, 400]" :page-size.sync="bookSourcePagination.size" layout="total, sizes, prev, pager, next" :total="bookSourceShowLength" :pager-count="collapseMenu ? 5 : 7"></el-pagination>
        </div>
      </div>
    </el-dialog>

    <el-dialog :title="'导入本地书籍' + importMultiBookTip" :visible.sync="showImportBookDialog" :width="dialogSmallWidth" :top="dialogTop" @closed="importBookDialogClosed" :fullscreen="collapseMenu" :class="isWebApp && !isNight ? 'status-bar-light-bg-dialog' : ''" v-if="$store.getters.isNormalPage">
      <div class="source-container table-container">
        <div class="check-form">
          <div class="book-cover">
            <el-image class="cover" :src="getCover(getBookCoverUrl(importBookInfo), true)" :key="getBookCoverUrl(importBookInfo)" fit="cover" lazy></el-image>
          </div>
          <div class="book-info">
            <div>
              <span>书名：</span>
              <el-input v-model="importBookInfo.name" size="small"></el-input>
            </div>
            <div>
              <span>作者：</span>
              <el-input v-model="importBookInfo.author" size="small"></el-input>
            </div>
            <div>
              <span>分组：</span>
              <el-select size="mini" v-model="importBookGroup" filterable multiple placeholder="未分组"><el-option v-for="(bookGroup, index) in bookGroupSetList" :key="'bookGroup-' + index" :label="bookGroup.groupName" :value="bookGroup.groupId"></el-option></el-select>
            </div>
            <div v-if="isShowTocRule">
              <span>规则：</span>
              <el-select size="mini" v-model="importUsedTxtRule" filterable placeholder="内置规则"><el-option v-for="(rule, index) in tocRuleList" :key="'txtTocRule-' + index" :label="rule.name" :value="rule.rule"></el-option></el-select>
              <el-button class="toc-refresh-btn" type="text" @click="getChapterListByRule()">刷新目录</el-button>
            </div>
            <div v-if="isShowTocRule">
              <el-input type="textarea" :rows="2" v-model="importBookInfo.tocUrl" size="small"></el-input>
            </div>
          </div>
        </div>
        <div class="chapter-title"> 章节列表({{ importBookChapters.length }}) </div>
        <div class="chapter-list" :style="{ maxHeight: dialogContentHeight - 40 - 35 + 'px' }">
          <p v-for="(chapter, index) in importBookChapters" :key="index"> {{ index + 1 }}. {{ chapter.title }} </p>
        </div>
      </div>
      <div slot="footer" class="dialog-footer">
        <el-button type="primary" size="medium" @click="saveBook(importBookInfo, true)">确定导入</el-button>
        <el-button size="medium" @click="showImportBookDialog = false">取消</el-button>
      </div>
    </el-dialog>

    <ShelfSettings v-model="showShelfSettingsDialog"></ShelfSettings>
  </div>
</template>


<script>
import Long from "long";
import { mapGetters } from "vuex";
import {
  defineComponent,
  ref,
  shallowRef,
  reactive,
  watch,
  onMounted,
  onBeforeUnmount
} from "vue";
import Axios from "../plugins/axios";
import { errorTypeList } from "../plugins/config";
import { setCache } from "../plugins/cache";
import eventBus from "../plugins/eventBus";
import { formatSize, LimitResquest } from "../plugins/helper";
const buildURL = require("axios/lib/helpers/buildURL");
import { isInContainer } from "element-ui/src/utils/dom";
import jump from "../plugins/jump";

/**
 * 书籍种类标签（书源搜索结果显示）
 */
const BookKind = {
  name: "BookKind",
  props: ["book"],
  methods: {
    renderBookKind() {
      if (!this.book) {
        return "";
      }
      const kindString = this.book.wordCount + "," + this.book.kind;
      if (!kindString) {
        return "";
      }
      const kindList = kindString.split(",");
      return kindList
        .filter(value => {
          return value && value !== "undefined";
        })
        .map(value => {
          return '<span class="small-tag">' + value + "</span>";
        })
        .join("");
    }
  },
  render(h) {
    return h("div", {
      staticClass: "book-kind",
      domProps: {
        innerHTML: this.renderBookKind(this.book)
      }
    });
  }
};

/**
 * 虚拟列表（列表优化）
 */
const VirtList = defineComponent({
  name: "VirtList",
  props: {
    list: { type: Array, default: () => [] },
    itemKey: { type: [String, Number], required: true },
    minSize: { type: Number, default: 20 },
    itemGap: { type: Number, default: 0 },
    renderControl: { type: Function, default: undefined },
    fixed: { type: Boolean, default: false },
    buffer: { type: Number, default: 0 },
    bufferTop: { type: Number, default: 0 },
    bufferBottom: { type: Number, default: 0 },
    scrollDistance: { type: Number, default: 0 },
    horizontal: { type: Boolean, default: false },
    start: { type: Number, default: 0 },
    offset: { type: Number, default: 0 },
    listStyle: { type: [String, Array, Object], default: "" },
    listClass: { type: [String, Array, Object], default: "" },
    itemStyle: { type: [String, Array, Object, Function], default: "" },
    itemClass: { type: [String, Array, Object, Function], default: "" }
  },
  emits: ["scroll", "toTop", "toBottom", "itemResize", "rangeUpdate"],
  setup(props, { emit }) {
    const clientRefEl = ref(null);
    const listRefEl = ref(null);
    const sizesMap = reactive(new Map());
    const reactiveData = reactive({
      scrollTop: 0,
      viewportSize: 0,
      clientSize: 0,
      renderBegin: 0,
      renderEnd: 0,
      virtualSize: 0,
      inViewBegin: 0,
      inViewEnd: 0,
      bufferTop: props.bufferTop || props.buffer,
      bufferBottom: props.bufferBottom || props.buffer
    });
    const renderList = shallowRef([]);
    let resizeObserver = null;

    const getItemSize = item => {
      if (!item) {
        return props.minSize;
      }
      return sizesMap.get(item[props.itemKey]) || props.minSize;
    };
    const getOffset = index => {
      let top = 0;
      for (let i = 0; i < index; i++) {
        top += getItemSize(props.list[i]) + props.itemGap;
      }
      return top;
    };
    const listTotalSize = () => {
      let total = 0;
      for (let i = 0; i < props.list.length; i++) {
        total += getItemSize(props.list[i]) + props.itemGap;
      }
      return total;
    };
    const updateRange = () => {
      const client = clientRefEl.value;
      if (!client) {
        return;
      }
      const scrollTop = client.scrollTop;
      const height = client.clientHeight || 0;
      reactiveData.scrollTop = scrollTop;
      reactiveData.viewportSize = height;
      reactiveData.clientSize = height;
      if (!props.list.length) {
        reactiveData.renderBegin = 0;
        reactiveData.renderEnd = -1;
        reactiveData.virtualSize = 0;
        renderList.value = [];
        return;
      }
      let begin = 0;
      let acc = 0;
      for (let i = 0; i < props.list.length; i++) {
        const size = getItemSize(props.list[i]) + props.itemGap;
        if (acc + size > scrollTop - props.minSize) {
          begin = i;
          break;
        }
        acc += size;
      }
      let end = begin;
      let acc2 = 0;
      for (let i = begin; i < props.list.length; i++) {
        acc2 += getItemSize(props.list[i]) + props.itemGap;
        end = i;
        if (acc2 > height + props.minSize * 2) {
          break;
        }
      }
      const inViewBegin = Math.max(0, begin - reactiveData.bufferTop);
      const inViewEnd = Math.min(
        props.list.length - 1,
        end + 2 + reactiveData.bufferBottom
      );
      reactiveData.inViewBegin = inViewBegin;
      reactiveData.inViewEnd = inViewEnd;
      if (props.renderControl) {
        const ctrl = props.renderControl(inViewBegin, inViewEnd);
        reactiveData.renderBegin = Math.max(0, ctrl.begin);
        reactiveData.renderEnd = Math.min(props.list.length - 1, ctrl.end);
      } else {
        reactiveData.renderBegin = inViewBegin;
        reactiveData.renderEnd = inViewEnd;
      }
      reactiveData.virtualSize = getOffset(reactiveData.renderBegin);
      renderList.value = props.list.slice(
        reactiveData.renderBegin,
        reactiveData.renderEnd + 1
      );
      emit("rangeUpdate", reactiveData.renderBegin, reactiveData.renderEnd);
    };
    const manualRender = (begin, end) => {
      const b = Math.max(0, begin);
      const e = Math.min(props.list.length - 1, end);
      reactiveData.renderBegin = b;
      reactiveData.renderEnd = e;
      reactiveData.virtualSize = getOffset(b);
      renderList.value = props.list.slice(b, e + 1);
    };
    const forceUpdate = () => {
      updateRange();
    };
    const reset = () => {
      reactiveData.renderBegin = 0;
      reactiveData.renderEnd = -1;
      reactiveData.virtualSize = 0;
      reactiveData.inViewBegin = 0;
      reactiveData.inViewEnd = 0;
      renderList.value = [];
    };
    const scrollToIndex = index => {
      const client = clientRefEl.value;
      if (!client || !props.list.length) {
        return;
      }
      const idx = Math.max(0, Math.min(index, props.list.length - 1));
      client.scrollTop = getOffset(idx);
      updateRange();
    };
    const scrollToTop = () => {
      scrollToIndex(0);
      emit("toTop", 0);
    };
    const scrollToBottom = () => {
      scrollToIndex(props.list.length - 1);
      emit("toBottom", props.list.length - 1);
    };
    const scrollToOffset = offset => {
      const client = clientRefEl.value;
      if (!client) {
        return;
      }
      client.scrollTop = offset;
      updateRange();
    };
    const scrollIntoView = index => {
      scrollToIndex(index);
    };
    const deleteItemSize = key => {
      sizesMap.delete(key);
    };
    const recordSize = (index, el) => {
      const item = props.list[index];
      if (!item || !el) {
        return;
      }
      const key = item[props.itemKey];
      const size = props.horizontal ? el.offsetWidth : el.offsetHeight;
      if (size && size !== sizesMap.get(key)) {
        sizesMap.set(key, size);
        updateRange();
      }
    };
    const getItemPosByIndex = index => {
      return {
        start: getOffset(index),
        end: getOffset(index) + getItemSize(props.list[index])
      };
    };

    watch(
      () => props.list.length,
      () => {
        if (props.list.length <= 0) {
          reset();
        } else {
          updateRange();
        }
      },
      { immediate: true }
    );

    onMounted(() => {
      const client = clientRefEl.value;
      if (client) {
        client.addEventListener("scroll", updateRange);
      }
      if (typeof ResizeObserver !== "undefined") {
        resizeObserver = new ResizeObserver(() => updateRange());
        if (client) {
          resizeObserver.observe(client);
        }
        if (listRefEl.value) {
          resizeObserver.observe(listRefEl.value);
        }
      }
      if (props.start) {
        scrollToIndex(props.start);
      } else if (props.offset) {
        scrollToOffset(props.offset);
      } else {
        updateRange();
      }
    });
    onBeforeUnmount(() => {
      const client = clientRefEl.value;
      if (client) {
        client.removeEventListener("scroll", updateRange);
      }
      if (resizeObserver) {
        resizeObserver.disconnect();
        resizeObserver = null;
      }
    });

    return {
      props,
      renderList,
      clientRefEl,
      listRefEl,
      reactiveData,
      sizesMap,
      resizeObserver,
      getOffset,
      listTotalSize,
      reset,
      scrollToIndex,
      manualRender,
      scrollIntoView,
      scrollToTop,
      scrollToBottom,
      scrollToOffset,
      getItemSize,
      deleteItemSize,
      forceUpdate,
      recordSize,
      getItemPosByIndex
    };
  },
  render() {
    const { renderList, reactiveData } = this;
    const {
      itemGap,
      itemKey,
      horizontal,
      listStyle,
      listClass,
      itemStyle,
      itemClass
    } = this.props;
    const slotFn = this.$scopedSlots.default;
    const h = this.$createElement;
    const items = renderList.map((item, i) => {
      const index = reactiveData.renderBegin + i;
      const style =
        typeof itemStyle === "function" ? itemStyle(item, index) : itemStyle;
      return h(
        "div",
        {
          key: item[itemKey],
          ref: "items",
          refInFor: true,
          class: [itemClass, "virt-list__item"],
          style: [style, itemGap ? `padding: ${itemGap / 2}px 0;` : ""],
          attrs: { "data-index": index }
        },
        slotFn ? slotFn({ itemData: item, index }) : []
      );
    });
    const sizeProp = horizontal ? "minWidth" : "minHeight";
    const padProp = horizontal ? "paddingLeft" : "paddingTop";
    const totalSize = this.listTotalSize();
    return h(
      "div",
      {
        ref: "clientRefEl",
        class: "virt-list__client",
        style: "width: 100%; height: 100%; overflow: auto;"
      },
      [
        h(
          "div",
          {
            ref: "listRefEl",
            class: ["virt-list__inner", listClass],
            style: [
              listStyle,
              {
                [sizeProp]: totalSize + "px",
                [padProp]: reactiveData.virtualSize + "px"
              }
            ]
          },
          items
        )
      ]
    );
  }
});

const BookList = {
  name: "BookList",
  components: { BookKind },
  data() {
    return {};
  },
  props: [
    "showNavigation",
    "bookList",
    "isSearchResult",
    "showBookEditButton",
    "showSourceCount"
  ],
  computed: {
    ...mapGetters(["dialogSmallWidth", "dialogTop"])
  },
  methods: {
    getBookCoverUrl(book) {
      return book.customCoverUrl || book.coverUrl;
    },
    dateFormat(t) {
      let time = new Date().getTime();
      let int = parseInt((time - t) / 1000);
      let str = "";
      if (int <= 30) {
        str = "刚刚";
      } else if (int < 60) {
        str = int + "秒前";
      } else if (int < 3600) {
        str = parseInt(int / 60) + "分钟前";
      } else if (int < 86400) {
        str = parseInt(int / 3600) + "小时前";
      } else if (int < 2592000) {
        str = parseInt(int / 86400) + "天前";
      } else if (int < 31536000) {
        str = parseInt(int / 2592000) + "月前";
      } else {
        str = parseInt(int / 31536000) + "年前";
      }
      return str;
    }
  },
  template: `
    <div class="wrapper">
      <div class="book" v-for="book in bookList" :key="book.bookUrl" :style="showNavigation ? { minWidth: '360px !important' } : {}">
        <div class="cover-img" @click.stop="$emit('showBookInfoDialog', book)">
          <el-image ref="bookCoverList" class="cover" :src="getCover(getBookCoverUrl(book), true)" fit="cover" lazy></el-image>
        </div>
        <div class="info" @click.stop="$emit('toDetail', book)">
          <div class="book-operation">
            <i class="el-icon-close" v-if="!isSearchResult && showBookEditButton" @click.stop="$emit('deleteBook', book)"></i>
            <i class="el-icon-edit" v-if="!isSearchResult && showBookEditButton" @click.stop="$emit('editBook', book)"></i>
            <i class="el-icon-edit" v-if="isSearchResult" @click.stop="$emit('editBook', book, true)"></i>
            <el-badge class="unread-num-badge" :max="99" :value="book.totalChapterNum - 1 - book.durChapterIndex" v-if="!isSearchResult && !showBookEditButton && book.totalChapterNum - 1 - book.durChapterIndex > 0" />
          </div>
          <div class="name" slot="reference" :class="showBookEditButton ? 'edit' : ''"> {{ book.name }} </div>
          <div class="sub">
            <div class="author"> {{ book.author || "" }} </div>
            <div class="dot" v-if="book.totalChapterNum">•</div>
            <div class="size" v-if="book.totalChapterNum"> 共{{ book.totalChapterNum }}章 </div>
          </div>
          <div class="dur-chapter" v-if="!isSearchResult && book.durChapterTitle"> 已读：{{ book.durChapterTitle }} </div>
          <BookKind v-if="isSearchResult" :book="book" />
          <div class="last-chapter" v-if="book.latestChapterTitle"> {{ book.lastCheckTime ? dateFormat(book.lastCheckTime) : "最新" }}：{{ book.latestChapterTitle }} </div>
          <div v-if="isSearchResult">
            <el-tag type="success" :effect="$store.getters.isNight ? 'dark' : 'light'" class="setting-connect" @click.stop="$emit('addBookToShelf', book)"> 加入书架 </el-tag>
            <span class="source-count" v-if="showSourceCount" @click.stop="$emit('showSearchBookSourceList', book)">{{ book.sourceCount || 1 }} 个书源</span>
          </div>
        </div>
      </div>
    </div>
  `
};

/**
 * 书架列表（虚拟列表视图）
 */
const BookVirtualList = {
  name: "BookList",
  components: { BookKind, VirtList },
  data() {
    return {};
  },
  props: [
    "showNavigation",
    "bookList",
    "isSearchResult",
    "showBookEditButton",
    "showSourceCount"
  ],
  computed: {
    ...mapGetters(["dialogSmallWidth", "dialogTop"])
  },
  methods: {
    getBookCoverUrl(book) {
      return book.customCoverUrl || book.coverUrl;
    },
    dateFormat(t) {
      let time = new Date().getTime();
      let int = parseInt((time - t) / 1000);
      let str = "";
      if (int <= 30) {
        str = "刚刚";
      } else if (int < 60) {
        str = int + "秒前";
      } else if (int < 3600) {
        str = parseInt(int / 60) + "分钟前";
      } else if (int < 86400) {
        str = parseInt(int / 3600) + "小时前";
      } else if (int < 2592000) {
        str = parseInt(int / 86400) + "天前";
      } else if (int < 31536000) {
        str = parseInt(int / 2592000) + "月前";
      } else {
        str = parseInt(int / 31536000) + "年前";
      }
      return str;
    }
  },
  template: `
    <div class="wrapper">
      <VirtList
        :item-key="'bookUrl'"
        :list="bookList"
        :min-size="112"
        item-class="book"
        :item-style="showNavigation ? { minWidth: '360px !important' } : {}"
      >
        <template slot-scope="{ itemData }">
          <div
            class="cover-img"
            @click.stop="$emit('showBookInfoDialog', itemData)"
          >
            <el-image
              ref="bookCoverList"
              class="cover"
              :src="getCover(getBookCoverUrl(itemData), true)"
              fit="cover"
              lazy
            >
            </el-image>
          </div>
          <div class="info" @click.stop="$emit('toDetail', itemData)">
            <div class="book-operation">
              <i
                class="el-icon-close"
                v-if="!isSearchResult && showBookEditButton"
                @click.stop="$emit('deleteBook', itemData)"
              ></i>
              <i
                class="el-icon-edit"
                v-if="!isSearchResult && showBookEditButton"
                @click.stop="$emit('editBook', itemData)"
              ></i>
              <i
                class="el-icon-edit"
                v-if="isSearchResult"
                @click.stop="$emit('editBook', itemData, true)"
              ></i>
              <el-badge
                class="unread-num-badge"
                :max="99"
                :value="
                  itemData.totalChapterNum - 1 - itemData.durChapterIndex
                "
                v-if="
                  !isSearchResult &&
                    !showBookEditButton &&
                    itemData.totalChapterNum - 1 - itemData.durChapterIndex > 0
                "
              />
            </div>
            <div class="name" slot="reference" :class="showBookEditButton ? 'edit' : ''"> {{ itemData.name }} </div>
            <div class="sub">
              <div class="author"> {{ itemData.author || "" }} </div>
              <div class="dot" v-if="itemData.totalChapterNum">•</div>
              <div class="size" v-if="itemData.totalChapterNum"> 共{{ itemData.totalChapterNum }}章 </div>
            </div>
            <div class="dur-chapter" v-if="!isSearchResult && itemData.durChapterTitle"> 已读：{{ itemData.durChapterTitle }} </div>
            <BookKind v-if="isSearchResult" :book="itemData" />
            <div class="last-chapter" v-if="itemData.latestChapterTitle"> {{ itemData.lastCheckTime ? dateFormat(itemData.lastCheckTime) : "最新" }}：{{ itemData.latestChapterTitle }} </div>
            <div v-if="isSearchResult">
              <el-tag type="success" :effect="$store.getters.isNight ? 'dark' : 'light'" class="setting-connect" @click.stop="$emit('addBookToShelf', itemData)"> 加入书架 </el-tag>
              <span class="source-count" v-if="showSourceCount" @click.stop="$emit('showSearchBookSourceList', itemData)">{{ itemData.sourceCount || 1 }} 个书源</span>
            </div>
          </div>
        </template>
      </VirtList>
    </div>
  `
};

/**
 * 书架列表（网格视图）
 */
const BookColumn = {
  name: "BookList",
  data() {
    return {
      containerWith: (this.$refs.wrapper || {}).clientWidth || 750
    };
  },
  props: ["showNavigation", "bookList", "isSearchResult", "showBookEditButton"],
  computed: {
    ...mapGetters(["dialogSmallWidth", "dialogTop"]),
    bookItemWidth() {
      const column = parseInt(
        this.$store.state.shelfConfig.viewCate.replace("column-", "")
      );
      if (column < 10) {
        return (this.containerWith - 15 * (column - 1) - 40) / column + "px";
      }
      return column + "px";
    }
  },
  mounted() {
    this.setWidth();
    this.resizeObserver = new ResizeObserver(() => {
      this.setWidth();
    });
    this.resizeObserver.observe(this.$refs.wrapper);
  },
  destroyed() {
    this.resizeObserver.disconnect();
    this.resizeObserver = null;
  },
  methods: {
    getBookCoverUrl(book) {
      return book.customCoverUrl || book.coverUrl;
    },
    setWidth() {
      this.containerWith = this.$refs.wrapper.clientWidth;
    },
    handleCoverClick(book, event) {
      if (!event.ctrlKey) {
        return null;
      }
      if (event.shiftKey || event.altKey || event.metaKey) {
        return null;
      }
      event.stopPropagation();
      this.$emit("showBookInfoDialog", book);
      return null;
    }
  },
  template: `
    <div
      class="wrapper"
      ref="wrapper"
      :style="{
        'grid-template-columns': 'repeat(auto-fill, ' + bookItemWidth + ')'
      }"
    >
      <div
        class="book"
        v-for="book in bookList"
        :key="book.bookUrl"
        :style="{ '--book-item-width': bookItemWidth }"
        @click="$emit('toDetail', book)"
      >
        <div class="cover-img" @click="handleCoverClick(book, $event)">
          <el-image
            ref="bookCoverList"
            class="cover"
            :src="getCover(getBookCoverUrl(book), true)"
            fit="cover"
            lazy
          >
          </el-image>
        </div>
        <div class="name"> {{ book.name }} </div>
        <div class="book-operation">
          <i
            class="el-icon-close"
            v-if="!isSearchResult && showBookEditButton"
            @click.stop="$emit('deleteBook', book)"
          ></i>
          <i
            class="el-icon-edit"
            v-if="!isSearchResult && showBookEditButton"
            @click.stop="$emit('editBook', book)"
          ></i>
          <i
            class="el-icon-edit"
            v-if="isSearchResult"
            @click.stop="$emit('editBook', book, true)"
          ></i>
          <el-badge
            class="unread-num-badge"
            :max="99"
            :value="book.totalChapterNum - 1 - book.durChapterIndex"
            v-if="
              !isSearchResult &&
                !showBookEditButton &&
                book.totalChapterNum - 1 - book.durChapterIndex > 0
            "
          />
        </div>
      </div>
    </div>
  `
};

/**
 * 虚拟列表下拉选择（单源搜索书源选择）
 */
const VirtualSelect = {
  name: "VirtualSelect",
  props: {
    width: { type: Number, default: 250 },
    size: { type: String, default: "small" },
    placeholder: { type: String, default: "请选择" },
    dataList: { type: Array, default: () => [] },
    value: { type: [String, Number], default: "" }
  },
  components: { VirtList },
  watch: {
    visibleVirtualList(val) {
      if (val) {
        this.keyword = "";
        this.$nextTick(() => {
          const index = this.curIndex ? this.curIndex : 0;
          this.$refs.virtList.scrollToIndex(index - 1);
        });
      } else {
        this.keyword = this.curLabel;
      }
    },
    value(val) {
      this.curValue = val;
      const item = this.dataList.find(v => v.bookSourceUrl === val);
      this.curLabel = (item && item.bookSourceName) || "";
      this.curIndex = this.dataList.findIndex(v => v.bookSourceUrl === val);
    },
    dataList(val) {
      const item = val.find(v => v.bookSourceUrl === this.value);
      this.curLabel = (item && item.bookSourceName) || "";
      this.curIndex = val.findIndex(v => v.bookSourceUrl === this.value);
    }
  },
  data() {
    return {
      visibleVirtualList: false,
      curValue: this.value,
      curLabel: "",
      curIndex: null,
      keyword: ""
    };
  },
  created() {
    this.$on("clickVirtualItem", item => {
      this.curIndex = item.index;
      this.curLabel = item.bookSourceName;
      this.visibleVirtualList = false;
      this.$emit("input", item.bookSourceUrl);
    });
  },
  computed: {
    list() {
      return this.dataList.filter(
        v => !this.keyword || v.bookSourceName.includes(this.keyword)
      );
    },
    tip() {
      return this.curLabel || this.placeholder || "请选择";
    }
  },
  mounted() {
    this.curValue = this.value;
    const item = this.dataList.find(v => v.bookSourceUrl === this.value);
    this.curLabel = (item && item.bookSourceName) || "";
    this.curIndex = this.dataList.findIndex(v => v.bookSourceUrl === this.value);
    this.keyword = this.curLabel;
  },
  methods: {
    handleInput(value) {
      this.keyword = value;
      this.$refs.virtList.$forceUpdate();
    }
  },
  template: `
    <el-popover
      popper-class="select-virtual-list-popover"
      trigger="click"
      placement="bottom-start"
      :width="width"
      v-model="visibleVirtualList"
      @show="$refs.virtList.$forceUpdate()"
    >
      <VirtList
        ref="virtList"
        :item-key="'bookUrl'"
        :list="list"
        :min-size="30"
      >
        <template slot-scope="{ itemData }">
          <div
            class="virtual-item"
            :class="{ 'is-selected': itemData.bookSourceUrl === curValue }"
            @click.stop="$emit('clickVirtualItem', itemData)"
          >
            <span>{{itemData.bookSourceName}}</span>
          </div>
        </template>
      </VirtList>
      <el-input
        slot="reference"
        :size="size"
        :placeholder="tip"
        :suffix-icon="
          visibleVirtualList ? 'el-icon-arrow-up' : 'el-icon-arrow-down'
        "
        v-model="keyword"
      ></el-input>
    </el-popover>
  `
};

/**
 * 探索书源（书海）
 */
const Explore = {
  name: "Explore",
  data() {
    return {
      page: 1,
      ruleFindUrl: "",
      bookSourceUrl: "",
      exploreList: [],
      showCollapse: [],
      sourceGroup: "全部",
      lastCollapseLength: 0
    };
  },
  props: ["visible", "bookSourceList"],
  computed: {
    theme() {
      return this.$store.getters.config.theme;
    },
    popupTheme() {
      return {
        background: this.$store.getters.currentThemeConfig.popup
      };
    },
    bookSourceListNew() {
      return this.bookSourceList.filter(v => v.exploreUrl);
    },
    bookSourceGroup() {
      const groups = new Set();
      groups.add("全部");
      this.bookSourceListNew.forEach(v => {
        if (v.bookSourceGroup) {
          v.bookSourceGroup.split(",").forEach(group => {
            if (group) {
              groups.add(group);
            }
          });
        }
      });
      groups.add("未分组");
      return Array.from(groups);
    },
    bookSourceShowLength() {
      if (this.sourceGroup && this.sourceGroup !== "全部") {
        return this.bookSourceListNew.filter(v => {
          if (this.sourceGroup === "未分组") {
            return !v.bookSourceGroup;
          }
          return (v.bookSourceGroup + ",").indexOf(this.sourceGroup + ",") >= 0;
        }).length;
      }
      return this.bookSourceListNew.length;
    }
  },
  mounted() {
    window.explorePop = this;
  },
  watch: {
    visible(val) {
      if (val) {
        this.$nextTick(() => {
          if (this.lastScrollTop) {
            this.$refs.sourceList.scrollTop = this.lastScrollTop;
          }
        });
      }
    }
  },
  methods: {
    onCollapseChange(names) {
      if (names.length > this.lastCollapseLength) {
        const source = this.bookSourceList[names[names.length - 1]];
        if (source.exploreUrl === true) {
          Axios.post(this.api + "/getBookSource", {
            bookSourceUrl: source.bookSourceUrl
          }).then(
            res => {
              if (res.data.isSuccess) {
                source.exploreUrl = res.data.data.exploreUrl;
                this.$set(source, "exploreGroup", this.getExploreGroup(source));
              }
            },
            error => {
              this.$message.error(
                "加载书源信息失败 " + (error && error.toString())
              );
            }
          );
        }
      }
      this.lastCollapseLength = names.length;
    },
    getExploreGroup(bookSource) {
      if (!bookSource.exploreUrl) {
        return [];
      }
      const result = [];
      let zone = [];
      let exploreUrlList = [];
      try {
        exploreUrlList = JSON.parse(bookSource.exploreUrl);
      } catch (error) {
        // 有些源的 key 是单引号的，尝试用 JS 解析
        try {
          exploreUrlList = new Function("return " + bookSource.exploreUrl)();
        } catch (error) {
          //
        }
      }
      if (Array.isArray(exploreUrlList) && exploreUrlList.length) {
        let percent = 0;
        exploreUrlList.forEach(v => {
          const basisPercent =
            (v.style && v.style.layout_flexBasisPercent) || 0.25;
          zone.push({
            name: v.title,
            url: v.url
          });
          percent += basisPercent;
          if (percent >= 1) {
            result.push(zone);
            zone = [];
            percent = 0;
          }
        });
      } else {
        bookSource.exploreUrl
          .replace(/\r\n/g, "\n")
          .split("\n")
          .forEach(v => {
            if (!v) {
              if (zone.length) {
                result.push(zone);
                zone = [];
              }
            } else {
              v = v.split("::");
              zone.push({
                name: v[0],
                url: v[1]
              });
            }
          });
      }
      if (zone.length) {
        result.push(zone);
      }
      return result;
    },
    exploreBookSource(url, sourceUrl, page) {
      this.page = page || 1;
      this.ruleFindUrl = url;
      this.bookSourceUrl = sourceUrl;
      Axios.post(this.api + "/exploreBook", {
        ruleFindUrl: url,
        bookSourceUrl: sourceUrl,
        page
      }).then(
        res => {
          if (res.data.isSuccess) {
            if (page === 1) {
              this.exploreList = res.data.data;
            } else {
              var data = [].concat(this.exploreList);
              var map = data.reduce((c, v) => {
                c[v.bookUrl] = v;
                return c;
              }, {});
              var length = data.length;
              res.data.data.forEach(v => {
                if (!map[v.bookUrl]) {
                  data.push(v);
                }
              });
              this.exploreList = data;
              if (data.length === length) {
                this.$message.error("没有更多啦");
              }
            }
            this.$emit("showSearchList", this.exploreList);
          }
        },
        error => {
          this.$message.error("探索失败 " + (error && error.toString()));
          throw error;
        }
      );
    },
    loadMore() {
      this.page = this.page + 1;
      this.exploreBookSource(this.ruleFindUrl, this.bookSourceUrl, this.page);
    },
    jumpToActive() {
      this.$nextTick(() => {
        let index = -1;
        this.bookSourceListNew.some((v, i) => {
          if (v.bookSourceUrl == this.bookSourceUrl) {
            index = i;
            return true;
          }
        });
        if (index < 0) {
          return;
        }
        let wrapper = this.$refs.sourceList;
        jump(this.$refs.source[index], {
          container: wrapper,
          duration: 0
        });
      });
    },
    close() {
      this.$emit("close");
    },
    scrollHandler() {
      this.lastScrollTop = this.$refs.sourceList.scrollTop;
    },
    setSourceGroup(group) {
      if (this.sourceGroup === group) {
        this.sourceGroup = "";
      } else {
        this.sourceGroup = group;
      }
    }
  },
  template: `
    <div class="popup-wrapper" :style="popupTheme">
      <div class="title-zone">
        <div class="title">书海</div>
        <div :class="{ 'title-btn': true }">
          <span class="source-count"> 共{{ bookSourceShowLength }}个可用书源 </span>
          <i
            class="el-icon-close close-btn"
            v-if="$store.state.miniInterface"
            @click.stop="close"
          ></i>
        </div>
      </div>
      <div class="source-group-wrapper">
        <el-tabs class="booksource-group-tabs" v-model="sourceGroup" stretch>
          <el-tab-pane
            v-for="group in bookSourceGroup"
            :key="'sourceGroupTab-' + group"
            :label="group"
            :name="group"
          ></el-tab-pane>
        </el-tabs>
      </div>
      <div
        class="data-wrapper"
        ref="sourceList"
        :class="{
          night: $store.getters.isNight,
          day: !$store.getters.isNight
        }"
        @scroll="scrollHandler"
      >
        <div class="cata">
          <el-collapse
            v-if="visible"
            class="source-collapse"
            ref="sourceList"
            v-model="showCollapse"
            @change="onCollapseChange"
          >
            <el-collapse-item
              v-for="(source, index) in bookSourceListNew"
              :key="'source-' + index"
              ref="source"
              :title="source.bookSourceName"
              :name="index"
              v-show="
                !sourceGroup ||
                  sourceGroup === '全部' ||
                  (sourceGroup === '未分组' && source.bookSourceGroup === '') ||
                  (source.bookSourceGroup + ',').indexOf(sourceGroup + ',') >= 0
              "
            >
              <div
                class="explore-group"
                v-if="showCollapse.includes(index)"
                v-for="(group, groupIndex) in source.exploreGroup"
                :key="'group-' + groupIndex"
              >
                <el-tag
                  v-for="(item, itemIndex) in group"
                  :key="'group-' + itemIndex"
                  class="explore-btn"
                  type="info"
                  :effect="$store.getters.isNight ? 'dark' : 'light'"
                  @click="
                    exploreBookSource(item.url, source.bookSourceUrl, 1, $event)
                  "
                > {{ item.name }} </el-tag>
              </div>
            </el-collapse-item>
          </el-collapse>
        </div>
      </div>
    </div>
  `
};

/**
 * 书架布局设置
 */
const ShelfSettings = {
  model: {
    prop: "show",
    event: "setShow"
  },
  name: "ShelfSettings",
  data() {
    return {
      isAdd: true,
      shelfConfig: {
        ...{
          showBookGroup: -1,
          viewCate: "list",
          bookOrder: "durChapterTime",
          imageProxy: "noProxy",
          virtualOptimize: "yes"
        },
        ...this.$store.state.shelfConfig
      },
      viewCateList: [
        { name: "列表", value: "list" },
        { name: "网格3列", value: "column-3" },
        { name: "网格4列", value: "column-4" },
        { name: "网格5列", value: "column-5" },
        { name: "网格6列", value: "column-6" },
        { name: "网格(小)", value: "column-100" },
        { name: "网格(中)", value: "column-150" },
        { name: "网格(大)", value: "column-200" }
      ],
      bookOrderList: [
        { name: "按阅读时间", value: "durChapterTime" },
        { name: "按更新时间", value: "lastCheckTime" },
        { name: "按书名", value: "name" }
      ],
      imageProxyList: [
        { name: "不使用代理", value: "noProxy" },
        { name: "服务器代理", value: "serverProxy" }
      ],
      virtualOptimizeList: [
        { name: "启用虚拟列表优化", value: "yes" },
        { name: "不启用优化", value: "no" }
      ]
    };
  },
  props: ["show"],
  computed: {
    ...mapGetters(["dialogSmallWidth", "dialogTop"]),
    bookGroupDisplayList() {
      return this.$store.state.bookGroupList
        .filter(v => this.getShowShelfBooks(v.groupId).length && v.show)
        .sort((a, b) => a.order - b.order);
    },
    shelfBooks() {
      return this.$store.getters.shelfBooks;
    }
  },
  watch: {
    show(val) {
      if (val) {
        this.shelfConfig = {
          ...{
            showBookGroup: -1,
            viewCate: "list",
            bookOrder: "durChapterTime",
            imageProxy: "noProxy",
            virtualOptimize: "yes"
          },
          ...this.$store.state.shelfConfig
        };
      }
    }
  },
  methods: {
    cancel() {
      this.$emit("setShow", false);
    },
    getShowShelfBooks(bookGroup) {
      if (bookGroup === -1) {
        return this.shelfBooks;
      } else if (bookGroup === -2) {
        return this.shelfBooks.filter(v => v.origin === "loc_book");
      } else if (bookGroup === -3) {
        return this.shelfBooks.filter(v => v.type === 1);
      } else if (bookGroup === -4) {
        return this.shelfBooks.filter(v => v.group === 0);
      } else if (bookGroup === -5) {
        return this.shelfBooks.filter(v => v.lastCheckError);
      }
      return this.shelfBooks.filter(v => {
        if (bookGroup === 0) {
          return true;
        }
        return Long.fromNumber(v.group || 0)
          .and(Long.fromNumber(bookGroup))
          .greaterThan(0);
      });
    },
    save() {
      this.$store.commit("setShelfConfig", {
        ...this.shelfConfig
      });
    }
  },
  template: `
    <el-dialog
      title="书架布局"
      :visible.sync="show"
      :width="dialogSmallWidth"
      :top="dialogTop"
      :fullscreen="$store.state.miniInterface"
      :before-close="cancel"
      :class="
        isWebApp && !$store.getters.isNight ? 'status-bar-light-bg-dialog' : ''
      "
    >
      <el-form :model="shelfConfig">
        <el-form-item label="显示分组">
          <el-select
            size="mini"
            v-model="shelfConfig.showBookGroup"
            class="setting-select"
            filterable
            placeholder="请选择默认显示分组"
          >
            <el-option
              v-for="(group, index) in bookGroupDisplayList"
              :key="'book-group-' + index"
              :label="group.groupName"
              :value="group.groupId"
            >
            </el-option>
          </el-select>
        </el-form-item>
        <el-form-item label="显示视图">
          <el-select
            size="mini"
            v-model="shelfConfig.viewCate"
            class="setting-select"
            filterable
            placeholder="请选择视图"
          >
            <el-option
              v-for="(view, index) in viewCateList"
              :key="'view-cate-' + index"
              :label="view.name"
              :value="view.value"
            >
            </el-option>
          </el-select>
        </el-form-item>
        <el-form-item label="书籍排序">
          <el-select
            size="mini"
            v-model="shelfConfig.bookOrder"
            class="setting-select"
            filterable
            placeholder="请选择排序"
          >
            <el-option
              v-for="(order, index) in bookOrderList"
              :key="'book-order-' + index"
              :label="order.name"
              :value="order.value"
            >
            </el-option>
          </el-select>
        </el-form-item>
        <el-form-item label="图片代理">
          <el-select
            size="mini"
            v-model="shelfConfig.imageProxy"
            class="setting-select"
            filterable
            placeholder="请选择图片代理"
          >
            <el-option
              v-for="(proxy, index) in imageProxyList"
              :key="'book-image-' + index"
              :label="proxy.name"
              :value="proxy.value"
            >
            </el-option>
          </el-select>
        </el-form-item>
        <el-form-item label="列表优化">
          <el-select
            size="mini"
            v-model="shelfConfig.virtualOptimize"
            class="setting-select"
            filterable
            placeholder="请选择是否启用列表优化"
          >
            <el-option
              v-for="(item, index) in virtualOptimizeList"
              :key="'virtual-options-' + index"
              :label="item.name"
              :value="item.value"
            >
            </el-option>
          </el-select>
        </el-form-item>
      </el-form>
      <div slot="footer" class="dialog-footer">
        <el-button size="medium" type="primary" @click="save"
          >保 存</el-button
        >
        <el-button size="medium" type="primary" @click="cancel"
          >关 闭</el-button
        >
      </div>
    </el-dialog>
  `
};

export default {
  components: {
    Explore,
    BookList,
    BookVirtualList,
    BookColumn,
    ShelfSettings,
    VirtualSelect
  },
  data() {
    return {
      search: "",
      searchTypeList: [
        { name: "单源搜索", value: "single" },
        { name: "多源搜索(过滤书名/作者名)", value: "multi" }
      ],
      isSearchResult: false,
      isExploreResult: false,
      searchResult: [],
      searchPage: 1,
      refreshLoading: false,
      searchLastIndex: -1,
      showBookEditButton: false,
      popExploreVisible: false,
      loadingMore: false,
      importSourceList: [],
      showImportSourceDialog: false,
      isImportRssSource: false,
      checkAll: false,
      isIndeterminate: false,
      checkedSourceIndex: [],
      showBookSourceManageDialog: false,
      manageSourceSelection: [],
      isShowFailureBookSource: false,
      checkBookSourceTip: "",
      isCheckingBookSource: false,
      showNavigation: false,
      navigationClass: "",
      navigationStyle: {},
      popIntroVisible: {},
      connecting: false,
      lastScrollTop: 0,
      localStorageAvaliable:
        window.localStorage &&
        window.localStorage.getItem &&
        window.localStorage.setItem,
      showSourceGroup: "全部",
      bookSourcePagination: {
        page: 1,
        size: 25
      },
      bookSourceKeyword: "",
      checkBookSourceConfig: {
        keyword: "斗罗大陆",
        timeout: 5000,
        concurrent: 5
      },
      importBookInfo: {},
      importBookGroup: [],
      importBookChapters: [],
      showImportBookDialog: false,
      importMultiBookTip: "",
      rssSource: {},
      concurrentList: [1, 2, 4, 8, 12, 18, 24, 30, 36, 42, 48, 54, 60],
      localCacheStats: {
        total: "0 Bytes",
        bookSourceList: "0 Bytes",
        rssSources: "0 Bytes",
        chapterList: "0 Bytes",
        chapterContent: "0 Bytes"
      },
      showLocalStoreManageDialog: false,
      showWebDAVManageDialog: false,
      importUsedTxtRule: "",
      showAddUser: false,
      addUserForm: {
        username: "",
        password: ""
      },
      isTauri: window.__TAURI__,
      keyword: "",
      showShelfSettingsDialog: false,
      searchBookSourceListMap: {},
      isShowSearchBookSourceListDesc: false
    };
  },
  watch: {
    searchConfig: {
      handler(val) {
        this.$store.commit("setSearchConfig", val);
        if (this.isSearchResult) {
          this.searchBook(1);
        }
      },
      deep: true
    },
    searchResult(val) {
      if (this.isSearchResult && val.length) {
        this.$nextTick(() => {
          this.$refs.bookList.scrollTop = this.lastScrollTop;
        });
      }
    },
    collapseMenu(val) {
      if (!val) {
        this.navigationClass = "";
      } else if (!this.showNavigation) {
        this.navigationClass = "navigation-hidden";
      }
    },
    showNavigation(val) {
      if (!val) {
        this.navigationClass = "navigation-out";
        setTimeout(() => {
          this.navigationClass = "navigation-hidden";
        }, 300);
      } else {
        this.navigationClass = "navigation-in";
      }
    },
    loginAuth() {
      this.init(true);
    },
    userNS() {
      this.init(true);
    },
    importUsedTxtRule(val) {
      if (val) {
        this.importBookInfo.tocUrl = val;
      }
    },
    importBookGroup(val) {
      if (val && this.showImportBookDialog) {
        let group = 0;
        val.forEach(v => {
          group = Long.fromNumber(group).or(Long.fromNumber(v)).toNumber();
        });
        this.importBookInfo.group = group;
      }
    },
    showBookGroup() {
      this.$nextTick(() => {
        setTimeout(this.ensureLoadBookCover);
      });
    },
    showBookEditButton(val) {
      if (!val) {
        this.keyword = "";
      }
    }
  },
  mounted() {
    document.title = "阅读";
    this.navigationClass =
      this.collapseMenu && !this.showNavigation ? "navigation-hidden" : "";
    window.shelfPage = this;
    eventBus.$on("onSourceFileChange", (event, isRssSource) => {
      if (this._inactive) {
        return;
      }
      this.onSourceFileChange(event, isRssSource);
    });
    eventBus.$on("editBook", (book, isAdd, onSuccess) => {
      if (this._inactive) {
        return;
      }
      this.editBook(book, isAdd, onSuccess);
    });
    eventBus.$on("importPreview", books => {
      if (!this._inactive) {
        this.importMultiBooks(books);
      }
    });
  },
  activated() {
    document.title = "阅读";
    this.scanCacheStorage();
    this.loadBookshelf();
    if (this.lastScrollTop) {
      this.$refs.bookList.scrollTop = 0 | this.lastScrollTop;
    }
  },
  methods: {
    init(refresh) {
      this.$root.$children[0].init(refresh);
    },
    setIP() {
      this.$prompt("请输入接口地址 ( 如：localhost:8080/reader3 )", "提示", {
        confirmButtonText: "确定",
        cancelButtonText: "取消",
        inputValue: this.api,
        beforeClose: (action, instance, done) => {
          if (action === "confirm") {
            this.connecting = true;
            instance.confirmButtonLoading = true;
            instance.confirmButtonText = "校验中……";
            var inputUrl = instance.inputValue.replace(/\/*$/g, "");
            this.loadBookshelf(inputUrl)
              .then(() => {
                this.connecting = false;
                instance.confirmButtonLoading = false;
                done();
                setCache("api_prefix", inputUrl);
                this.$store.commit("setApi", inputUrl);
                this.init();
              })
              .catch(() => {
                instance.confirmButtonLoading = false;
                instance.confirmButtonText = "确定";
              });
          } else {
            done();
          }
        }
      })
        .then(({ value }) => {
          this.$message({
            type: "success",
            message: "与" + value + "连接成功"
          });
        })
        .catch(() => {});
    },
    loadBookshelf(api, refresh) {
      api = api || this.api;
      if (!api) {
        this.$message.error("请先设置后端接口地址");
        this.$store.commit("setConnected", false);
        return Promise.reject(false);
      }

      if (
        !(this.shelfBooks && this.shelfBooks.length) &&
        !(this.loading && this.loading.visible)
      ) {
        this.loading = this.$loading({
          target: this.$refs.bookList,
          lock: true,
          text: refresh ? "正在刷新书籍信息" : "正在获取书籍信息",
          spinner: "el-icon-loading",
          background: this.isNight ? "#222" : "#fff"
        });
      }
      this.refreshLoading = true;

      if (
        !api.startsWith("http://") &&
        !api.startsWith("https://") &&
        !api.startsWith("//")
      ) {
        api = "//" + api;
      }

      return this.$root.$children[0].loadBookShelf(refresh, api).then(() => {
        this.refreshLoading = false;
        if (this.loading) {
          this.loading.close();
        }
      });
    },
    refreshShelf() {
      return this.loadBookshelf(null, true);
    },
    loadBookGroup(refresh) {
      return this.$root.$children[0].loadBookGroup(refresh);
    },
    loadBookSource(refresh) {
      return this.$root.$children[0].loadBookSource(refresh);
    },
    searchBook(page) {
      if (!this.$store.state.connected) {
        this.$message.error("后端未连接");
        return;
      }
      if (!this.search) {
        this.$message.error("请输入关键词进行搜索");
        return;
      }
      if (
        this.searchConfig.searchType === "single" &&
        !this.searchConfig.bookSourceUrl
      ) {
        this.$message.error("请选择书源进行搜索");
        return;
      }
      if (page) {
        this.searchPage = page;
      }
      page = this.searchPage;
      if (page === 1) {
        // 重新搜索
        this.searchLastIndex = -1;
        this.searchBookSourceListMap = {};
      }
      if (this.searchConfig.searchType === "multi" && window.EventSource) {
        this.searchBookByEventStream(page);
        return;
      }
      if (this.loadingMore) {
        return;
      }
      this.isSearchResult = true;
      this.isExploreResult = false;
      this.loadingMore = true;
      if (page === 1) {
        this.searchResult = [];
      }
      Axios.post(
        this.api +
          (this.searchConfig.searchType === "single"
            ? "/searchBook"
            : "/searchBookMulti"),
        {
          key: this.search,
          bookSourceUrl: this.searchConfig.bookSourceUrl,
          bookSourceGroup: this.searchConfig.bookSourceGroup,
          concurrentCount: this.searchConfig.concurrentCount,
          lastIndex: this.searchLastIndex, // 多源搜索时的索引
          page: page // 单源搜索时的page
        },
        {
          timeout: this.searchConfig.searchType === "single" ? 30000 : 180000
        }
      ).then(
        res => {
          this.loadingMore = false;
          if (res.data.isSuccess) {
            //
            let resultList = [];
            if (this.searchConfig.searchType === "single") {
              resultList = res.data.data;
            } else {
              this.searchLastIndex = res.data.data.lastIndex;
              resultList = res.data.data.list;
            }
            var data = [].concat(this.searchResult);
            var sourceMap = data.reduce((map, item) => {
              map[item.name + "_" + item.author] = item;
              return map;
            }, {});
            var length = data.length;
            resultList.forEach(v => {
              if (!this.searchResultMap[v.bookUrl]) {
                const sourceKey = v.name + "_" + v.author;
                if (sourceMap[sourceKey]) {
                  sourceMap[sourceKey].sourceCount =
                    sourceMap[sourceKey].sourceCount || 1;
                  sourceMap[sourceKey].sourceCount += 1;
                } else {
                  v.sourceCount = 1;
                  sourceMap[sourceKey] = v;
                  data.push(v);
                }
                if (!this.searchBookSourceListMap[sourceKey]) {
                  this.searchBookSourceListMap[sourceKey] = [];
                }
                this.searchBookSourceListMap[sourceKey].push(v);
              }
            });
            this.searchResult = data;
            if (data.length === length) {
              this.$message.error("没有更多啦");
            }
          }
        },
        error => {
          this.$message.error("搜索书籍失败 " + (error && error.toString()));
        }
      );
    },
    searchBookByEventStream(page) {
      const tryClose = () => {
        try {
          if (
            this.searchEventSource &&
            this.searchEventSource.readyState != this.searchEventSource.CLOSED
          ) {
            this.searchEventSource.close();
          }
          this.searchEventSource = null;
        } catch (error) {
          //
        }
      };
      if (this.loadingMore) {
        tryClose();
        this.loadingMore = false;
        // page === 1 是重新搜索
        if (page !== 1) {
          // 停止搜索
          return;
        }
      }
      const params = {
        accessToken: this.$store.state.token,
        key: this.search,
        bookSourceUrl: this.searchConfig.bookSourceUrl,
        bookSourceGroup: this.searchConfig.bookSourceGroup,
        concurrentCount: this.searchConfig.concurrentCount,
        lastIndex: this.searchLastIndex, // 多源搜索时的索引
        page: page // 单源搜索时的page
      };

      this.isSearchResult = true;
      this.isExploreResult = false;
      this.loadingMore = true;
      if (page === 1) {
        this.searchResult = [];
      }
      const url = buildURL(this.api + "/searchBookMultiSSE", params);

      tryClose();

      this.searchEventSource = new EventSource(url, {
        withCredentials: true
      });
      this.searchEventSource.addEventListener("error", e => {
        this.loadingMore = false;
        tryClose();
        try {
          if (e.data) {
            const result = JSON.parse(e.data);
            if (result && result.errorMsg) {
              this.$message.error(result.errorMsg);
            }
          }
        } catch (error) {
          //
        }
      });
      let oldSearchResultLength = this.searchResult.length;
      this.searchEventSource.addEventListener("end", e => {
        this.loadingMore = false;
        tryClose();
        try {
          let isEnd = false;
          if (e.data) {
            const result = JSON.parse(e.data);
            if (result && result.lastIndex) {
              this.searchLastIndex = result.lastIndex;
            }
            if (result && result.isEnd) {
              isEnd = true;
            }
          }
          if (this.searchResult.length === oldSearchResultLength) {
            this.$message.error(isEnd ? "没有更多啦" : "本次未搜索到数据");
          }
        } catch (error) {
          //
        }
      });
      this.searchEventSource.addEventListener("message", e => {
        try {
          if (e.data) {
            const result = JSON.parse(e.data);
            if (result && result.lastIndex) {
              this.searchLastIndex = result.lastIndex;
            }
            if (result.data) {
              var data = [].concat(this.searchResult);
              var sourceMap = data.reduce((map, item) => {
                map[item.name + "_" + item.author] = item;
                return map;
              }, {});
              result.data.forEach(v => {
                if (!this.searchResultMap[v.bookUrl]) {
                  const sourceKey = v.name + "_" + v.author;
                  if (sourceMap[sourceKey]) {
                    sourceMap[sourceKey].sourceCount =
                      sourceMap[sourceKey].sourceCount || 1;
                    sourceMap[sourceKey].sourceCount += 1;
                  } else {
                    v.sourceCount = 1;
                    sourceMap[sourceKey] = v;
                    data.push(v);
                  }
                  if (!this.searchBookSourceListMap[sourceKey]) {
                    this.searchBookSourceListMap[sourceKey] = [];
                  }
                  this.searchBookSourceListMap[sourceKey].push(v);
                }
              });
              this.searchResult = data.sort((a, b) => {
                if (a.name === this.search) {
                  return -1;
                } else if (b.name === this.search) {
                  return 1;
                } else if (a.name.indexOf(this.search) >= 0) {
                  return -1;
                } else if (a.name.indexOf(this.search) >= 0) {
                  return 1;
                } else if (a.sourceCount > b.sourceCount) {
                  return -1;
                }
                return 1;
              });
            }
          }
        } catch (error) {
          //
        }
      });
    },
    searchBookManual() {
      this.$prompt("请输入书籍链接", "提示", {
        confirmButtonText: "确定",
        cancelButtonText: "取消",
        inputValue: this.search,
        inputPattern: /^(http|https):\/\/.+$/,
        inputErrorMessage: "url 形式不正确"
      })
        .then(({ value }) => this.searchBookDirectly(value))
        .catch(() => {});
    },
    searchBookDirectly(url) {
      this.isSearchResult = true;
      this.isExploreResult = false;
      this.loadingMore = true;
      Axios.post(
        this.api + "/getBookInfo",
        { url, bookSourceUrl: this.searchConfig.bookSourceUrl },
        { timeout: 30000 }
      ).then(
        res => {
          this.loadingMore = false;
          if (res.data.isSuccess) {
            if (res.data.data && res.data.data.name && res.data.data.author) {
              this.searchResult = [res.data.data];
            } else {
              this.$message.error("没有搜索到书籍");
            }
          }
        },
        error => {
          this.$message.error("搜索书籍失败 " + (error && error.toString()));
        }
      );
    },
    saveBookManual() {
      const book = {
        bookUrl: "书籍链接",
        tocUrl: "目录链接",
        origin: this.searchConfig.bookSourceUrl,
        originName: "花生小说",
        name: "书名",
        author: "作者"
      };
      this.editBook(book, true, () => this.loadBookshelf());
    },
    showSearchBookSourceList(book) {
      this.oldSearchResult = [].concat(this.searchResult);
      this.searchResult =
        this.searchBookSourceListMap[book.name + "_" + book.author] || [];
      this.isShowSearchBookSourceListDesc = true;
    },
    displaySearchResult() {
      this.searchResult = this.oldSearchResult;
      this.isShowSearchBookSourceListDesc = false;
    },
    toDetail(book) {
      if (!book.bookUrl) {
        return;
      }
      this.$store.commit("setReadingBook", {
        ...book,
        name: book.name,
        bookUrl: book.bookUrl,
        index: book.index ?? book.durChapterIndex ?? 0,
        type: book.type,
        coverUrl: this.getBookCoverUrl(book),
        tocUrl: book.tocUrl,
        author: book.author,
        origin: book.origin,
        originName: book.originName,
        latestChapterTitle: book.latestChapterTitle,
        intro: book.intro
      });
      this.$router.push({
        path: "/reader",
        query: this.isSearchResult
          ? { search: 1, bookUrl: book.bookUrl }
          : { bookUrl: book.bookUrl }
      });
    },
    async addBookToShelf(book) {
      const bookInfo = { ...book };
      const customImportBookInfo = await this.customImportBookInfo(
        {
          title: "修改书籍",
          cancelButtonText: "暂不加入"
        },
        bookInfo
      );
      if (customImportBookInfo === false) {
        return;
      }
      this.saveBook({ ...bookInfo, ...customImportBookInfo });
    },
    saveBook(book, isImport, isEdit) {
      if (!book || !book.bookUrl || !book.origin) {
        this.$message.error("书籍信息错误");
        return Promise.reject(false);
      }
      return Axios.post(this.api + "/saveBook", book).then(
        res => {
          if (res.data.isSuccess) {
            //
            if (isImport) {
              this.showImportBookDialog = false;
            }
            this.$message.success(
              isImport
                ? "导入书籍成功"
                : isEdit
                ? "修改书籍成功"
                : "加入书架成功"
            );
            if (isEdit) {
              this.$store.commit("updateShelfBook", res.data.data);
            } else {
              this.loadBookshelf();
            }
            return res.data.data;
          }
        },
        error => {
          this.$message.error(
            (isImport
              ? "导入书籍失败"
              : isEdit
              ? "修改书籍失败"
              : "加入书架失败 ") + (error && error.toString())
          );
        }
      );
    },
    deleteBook(book) {
      this.$root.$children[0].deleteBook(book);
    },
    editBook(book, isAdd, onSuccess) {
      if (!book || !book.name || !book.bookUrl || !book.origin) {
        this.$message.error("书籍信息错误");
        return;
      }
      const bookInfo = { ...book };
      delete bookInfo["variableMap$delegate"];
      eventBus.$emit(
        "showEditor",
        isAdd ? "保存书籍" : "编辑书籍",
        JSON.stringify(bookInfo, null, 4),
        async (content, close) => {
          try {
            const newBook = JSON.parse(content);
            if (!newBook.name) {
              this.$message.error("书籍名称不能为空");
              return;
            }
            if (!newBook.bookUrl) {
              this.$message.error("书籍链接不能为空");
              return;
            }
            if (!newBook.origin) {
              this.$message.error("书籍来源不能为空");
              return;
            }
            if (isAdd) {
              const res = await this.$confirm(
                "加入书架之后才能编辑书籍信息, 是否加入书架?",
                "提示",
                {
                  confirmButtonText: "确定",
                  cancelButtonText: "取消",
                  type: "warning"
                }
              ).catch(() => {
                return false;
              });
              if (!res) {
                return;
              }
            }
            this.saveBook(newBook, false, true).then(() => {
              close();
              if (onSuccess) {
                onSuccess();
              }
            });
          } catch (e) {
            this.$message.error("书籍信息必须是JSON格式");
          }
        }
      );
    },
    currentDateTime() {
      const now = new Date();
      const pad = a => (a < 10 ? "0" + a : a);
      return (
        now.getFullYear() +
        pad(now.getMonth() + 1) +
        pad(now.getDate()) +
        "_" +
        pad(now.getHours()) +
        pad(now.getMinutes()) +
        pad(now.getSeconds())
      );
    },
    backToShelf() {
      this.isSearchResult = false;
      this.isExploreResult = false;
      this.searchResult = [];
      this.loadingMore = false;
    },
    toogleNight() {
      if (this.isNight) {
        this.$store.commit("setNightTheme", false);
      } else {
        this.$store.commit("setNightTheme", true);
      }
    },
    showSearchList(data) {
      this.isSearchResult = true;
      this.isExploreResult = true;
      this.loadingMore = false;
      this.searchResult = data;
    },
    loadMore() {
      this.lastScrollTop = this.$refs.bookList.scrollTop;
      if (this.isExploreResult) {
        this.loadingMore = true;
        this.$refs.popExplore.loadMore();
      } else {
        this.searchBook(this.searchPage + 1);
      }
    },
    uploadBookSource() {
      this.$refs.fileRef.dispatchEvent(new MouseEvent("click"));
    },
    onSourceFileChange(event, isRssSource) {
      const rawFile = event.target.files && event.target.files[0];
      const reader = new FileReader();
      const sourceTypeName = isRssSource ? "RSS源" : "书源";
      reader.onload = e => {
        const data = e.target.result;
        try {
          const sourceList = JSON.parse(data);
          if (Array.isArray(sourceList) && sourceList.length) {
            this.importSourceList = sourceList.map(v => {
              if (v.headerMap) {
                if (!v.header) {
                  v.header =
                    typeof v.headerMap === "string"
                      ? v.headerMap
                      : JSON.stringify(v.headerMap);
                }
                delete v.headerMap;
              }
              return v;
            });
            this.showImportSourceDialog = true;
            this.isImportRssSource = !!isRssSource;
          } else {
            this.$message.error(sourceTypeName + "文件错误");
          }
        } catch (error) {
          this.$message.error(sourceTypeName + "文件错误");
        }
      };
      reader.onerror = () => {
        // FileReader 读取出错，只能上传读取了
        let param = new FormData();
        param.append("file", rawFile);
        Axios.post(this.api + "/readSourceFile", param, {
          headers: { "Content-Type": "multipart/form-data" }
        }).then(
          res => {
            if (res.data.isSuccess) {
              //
              let sourceList = [];
              res.data.data.forEach(v => {
                try {
                  const data = JSON.parse(v);
                  if (Array.isArray(data)) {
                    sourceList = sourceList.concat(data);
                  }
                } catch (error) {
                  //
                }
              });
              if (sourceList.length) {
                this.importSourceList = sourceList.map(v => {
                  if (v.headerMap) {
                    if (!v.header) {
                      v.header =
                        typeof v.headerMap === "string"
                          ? v.headerMap
                          : JSON.stringify(v.headerMap);
                    }
                    delete v.headerMap;
                  }
                  return v;
                });
                this.showImportSourceDialog = true;
                this.isImportRssSource = !!isRssSource;
              } else {
                this.$message.error(sourceTypeName + "文件错误");
              }
            }
          },
          error => {
            this.$message.error(
              "读取" +
                sourceTypeName +
                "文件内容失败 " +
                (error && error.toString())
            );
          }
        );
      };
      reader.readAsText(rawFile);
      this.$refs.fileRef.value = null;
    },
    async showRemoteBookSourceSubDialog() {
      eventBus.$emit("showRemoteBookSourceSubDialog");
    },
    handleCheckAllChange(val) {
      let hasFilterd = false;
      this.checkedSourceIndex = val
        ? this.importSourceList
            .map((v, i) => {
              // 不勾选使用了 js，webview的书源
              const source = JSON.stringify(v);
              if (
                source.indexOf("@js:") !== -1 ||
                source.match(/[\\\"]*webView[\\\"]*:/)
              ) {
                hasFilterd = true;
                return false;
              }
              return i;
            })
            .filter(v => v !== false)
        : [];
      if (val && hasFilterd) {
        this.$message.info("部分使用了Javascript和Webview的书源未勾选");
      }
      this.isIndeterminate = false;
    },
    handleCheckedSourcesChange(value) {
      let checkedCount = value.length;
      this.checkAll = checkedCount === this.importSourceList.length;
      this.isIndeterminate =
        checkedCount > 0 && checkedCount < this.importSourceList.length;
    },
    handleCheckWebviewChange(value) {
      this.checkedSourceIndex = this.importSourceList
        .map((source, index) => {
          const sourceText = JSON.stringify(source);
          return sourceText.match(/[\\\"]*webView[\\\"]*:/)
            ? value && index
            : this.checkedSourceIndex.includes(index) && index;
        })
        .filter(index => index !== false);
    },
    handleCheckJSChange(value) {
      this.checkedSourceIndex = this.importSourceList
        .map((source, index) => {
          const sourceText = JSON.stringify(source);
          return sourceText.indexOf("@js:") !== -1
            ? value && index
            : this.checkedSourceIndex.includes(index) && index;
        })
        .filter(index => index !== false);
    },
    getSourceTag(source) {
      const sourceStr = JSON.stringify(source);
      const tags = [];
      if (sourceStr.indexOf("@js:") !== -1) {
        tags.push("@Javascript");
      }

      if (sourceStr.match(/[\\\"]*webView[\\\"]*:/)) {
        tags.push("@WebView");
      }

      return "   " + tags.join("  ");
    },
    saveSourceList() {
      if (!this.$store.state.connected) {
        this.$message.error("后端未连接");
        return;
      }
      if (!this.checkedSourceIndex.length) {
        this.$message.error("请选择需要导入的源");
        return;
      }
      const sourceList = this.checkedSourceIndex.map(
        v => this.importSourceList[v]
      );
      Axios.post(
        this.api +
          (this.isImportRssSource ? "/saveRssSources" : "/saveBookSources"),
        sourceList,
        { timeout: 300000 }
      ).then(
        res => {
          if (res.data.isSuccess) {
            //
            this.$message.success(
              this.isImportRssSource ? "导入RSS源成功" : "导入书源成功"
            );
            if (this.isImportRssSource) {
              this.loadRssSources(true);
            } else {
              this.loadBookSource(true);
            }
            this.showImportSourceDialog = false;
            this.isImportRssSource = false;
            this.checkedSourceIndex = [];
          }
        },
        error => {
          this.$message.error(
            (this.isImportRssSource ? "导入RSS源失败 " : "导入书源失败 ") +
              (error && error.toString())
          );
        }
      );
    },
    isBookSourceSelectable(bookSource) {
      const res = [];
      (this.$store.state.shelfBooks || []).forEach(v => {
        if (v.origin === bookSource.bookSourceUrl) {
          res.push(v.name);
        }
      });
      return !res.length;
    },
    showSourceBook(bookSource) {
      const res = [];
      (this.$store.state.shelfBooks || []).forEach(v => {
        if (v.origin === bookSource.bookSourceUrl) {
          res.push(v.name);
        }
      });
      return res.join("\n");
    },
    getInvalidBookSources() {
      if (!this.$store.state.connected) {
        this.$message.error("后端未连接");
        return;
      }
      Axios.post(this.api + "/getInvalidBookSources").then(
        res => {
          if (res.data.isSuccess) {
            //
            res.data.data.forEach(v => {
              this.$store.commit("addFailureBookSource", {
                bookSourceUrl: v.sourceUrl,
                errorMsg: v.error
              });
            });
          }
        },
        () => {
          //
        }
      );
    },
    async checkBookSource() {
      if (this.isCheckingBookSource) {
        const cancel = await this.$confirm(
          "正在检测失效书源, 是否终止?",
          "提示",
          {
            confirmButtonText: "确定",
            cancelButtonText: "取消",
            type: "warning"
          }
        ).catch(() => false);
        if (cancel) {
          this.bookSourceChecker.cancel();
        }
        return;
      }
      if (!this.checkBookSourceConfig.keyword) {
        this.$message.error("请输入搜索关键词");
        return;
      }
      this.isCheckingBookSource = true;
      this.$store.commit("setFailureIncludeTimeout", true);
      this.bookSourceChecker = LimitResquest(
        this.checkBookSourceConfig.concurrent,
        handler => {
          this.checkBookSourceTip =
            handler.requestCount + "/" + this.bookSourceList.length;
          if (handler.isEnd()) {
            this.isCheckingBookSource = false;
            this.$store.commit("setFailureIncludeTimeout", false);
          }
        }
      );
      this.bookSourceList.forEach(v => {
        this.bookSourceChecker(() => {
          return Axios.post(
            this.api + "/searchBook",
            {
              key: this.checkBookSourceConfig.keyword,
              bookSourceUrl: v.bookSourceUrl
            },
            {
              timeout: this.checkBookSourceConfig.timeout,
              silent: true
            }
          );
        });
      });
    },
    async deleteBookSourceList() {
      if (!this.manageSourceSelection.length) {
        this.$message.error("请选择需要删除的源");
        return;
      }
      const res = await this.$confirm("确认要删除所选择的书源吗?", "提示", {
        confirmButtonText: "确定",
        cancelButtonText: "取消",
        type: "warning"
      }).catch(() => {
        return false;
      });
      if (!res) {
        return;
      }
      Axios.post(
        this.api + "/deleteBookSources",
        this.manageSourceSelection
      ).then(
        res => {
          if (res.data.isSuccess) {
            this.$store.commit(
              "removeFailureBookSource",
              this.manageSourceSelection
            );
            this.manageSourceSelection = [];
            this.$message.success("删除书源成功");
            this.loadBookSource(true);
          }
        },
        error => {
          this.$message.error("删除书源失败 " + (error && error.toString()));
        }
      );
    },
    hideMenu() {
      if (this.$store.getters.isNormalPage && this.collapseMenu) {
        this.showNavigation = false;
      }
    },
    toggleMenu() {
      if (this.collapseMenu) {
        this.showNavigation = !this.showNavigation;
      }
    },
    showExplorePop() {
      setTimeout(() => {
        this.popExploreVisible = true;
      }, 100);
    },
    showBookInfoDialog(book) {
      eventBus.$emit("showBookInfoDialog", book);
    },
    saveUserConfig() {
      this.$root.$children[0].saveUserConfig();
    },
    restoreUserConfig() {
      this.$root.$children[0].restoreUserConfig();
    },
    loadUserList() {
      if (!this.$store.state.connected) {
        this.$message.error("后端未连接");
        return;
      }
      Axios.get(this.api + "/getUserList").then(
        res => {
          if (res.data.isSuccess) {
            this.userNS = this.$store.state.userInfo.username;
            this.userList = res.data.data.map(v => ({
              ...v,
              userNS: v.username
            }));
            this.$store.commit("setIsManagerMode", true);
          }
        },
        error => {
          this.$message.error(
            "加载用户空间失败 " + (error && error.toString())
          );
        }
      );
    },
    formatTableField(row, column, cellValue) {
      switch (column.property) {
        case "createdAt":
        case "lastLoginAt":
        case "lastModified":
          return cellValue ? new Date(cellValue).format("yy-MM-dd hh:mm") : "";
        case "size":
          return row.isDirectory ? "" : formatSize(cellValue);
        default:
          return cellValue;
      }
    },
    exitSecureMode() {
      this.userNS = "default";
      this.userList = [];
      this.$store.commit("setIsManagerMode", false);
      this.init(true);
    },
    async backupToWebdav() {
      const res = await this.$confirm(
        "确认要用当前数据覆盖备份文件中的书源、书架、分组、RSS订阅数据、替换规则、书签、用户配置和Webdav书籍吗?",
        "提示",
        {
          confirmButtonText: "确定",
          cancelButtonText: "取消",
          type: "warning"
        }
      ).catch(() => {
        return false;
      });
      if (!res) {
        return;
      }
      Axios.post(this.api + "/backupToWebdav").then(
        res => {
          if (res.data.isSuccess) {
            this.$message.success("备份成功");
          }
        },
        error => {
          this.$message.error("备份失败 " + (error && error.toString()));
        }
      );
    },
    handleTouchStart(e) {
      this.lastTouch = false;
      this.lastMoveX = false;
      this.touchMoveTimes = 0;
      // 边缘 20px 以内禁止触摸
      if (
        e.touches &&
        e.touches[0] &&
        e.touches[0].clientX > 20 &&
        e.touches[0].clientX < window.innerWidth - 20 &&
        e.touches[0].clientY > 20 &&
        e.touches[0].clientY < window.innerHeight - 20
      ) {
        this.lastTouch = e.touches[0];
      }
    },
    handleTouchMove(e) {
      if (e.touches && e.touches[0] && this.lastTouch && this.collapseMenu) {
        const moveX = e.touches[0].clientX - this.lastTouch.clientX;
        const moveY = e.touches[0].clientY - this.lastTouch.clientY;
        if (Math.abs(moveY) > Math.abs(moveX)) {
          this.navigationStyle = {};
          this.lastMoveX = 0;
          return;
        }
        e.preventDefault();
        e.stopPropagation();
        if (!this.showNavigation && moveX > 0 && moveX <= 270) {
          // 往右拉，打开目录
          if (this.touchMoveTimes % 3 === 0) {
            this.navigationStyle = {
              marginLeft: moveX - 270 + "px"
            };
          }
          this.lastMoveX = moveX;
        } else if (this.showNavigation && moveX < 0 && moveX >= -270) {
          // 往左拉，关闭目录
          if (this.touchMoveTimes % 3 === 0) {
            this.navigationStyle = {
              marginLeft: moveX + "px"
            };
          }
          this.lastMoveX = moveX;
        }
        this.touchMoveTimes++;
      }
    },
    handleTouchEnd() {
      if (this.collapseMenu) {
        if (this.lastMoveX > 0) {
          this.showNavigation = true;
          this.navigationStyle = {};
        } else if (this.lastMoveX < 0) {
          this.showNavigation = false;
          this.navigationStyle = {};
        }
      }
    },
    showFailureBookSource() {
      this.getInvalidBookSources();
      this.isShowFailureBookSource = true;
      this.showBookSourceManageDialog = true;
    },
    debugBookSource() {
      window.open(
        window.location.origin +
          window.location.pathname.replace(/index.html$/, "") +
          "bookSourceDebug/#domain=" +
          this.api,
        "_blank"
      );
    },
    setShowSourceGroup(group) {
      if (this.showSourceGroup === group) {
        this.showSourceGroup = "";
      } else {
        this.showSourceGroup = group;
      }
    },
    importLocalBook() {
      this.$refs.bookRef.dispatchEvent(new MouseEvent("click"));
    },
    onBookFileChange(event) {
      if (!event.target || !event.target.files || !event.target.files.length) {
        return;
      }
      let param = new FormData();
      for (let i = 0; i < event.target.files.length; i++) {
        const file = event.target.files[i];
        param.append("file" + i, file);
      }
      Axios.post(this.api + "/importBookPreview", param, {
        headers: { "Content-Type": "multipart/form-data" }
      }).then(
        res => {
          if (res.data.isSuccess && res.data.data.length) {
            if (res.data.data.length > 1) {
              // 批量导入
              this.importMultiBooks(res.data.data);
            } else {
              //
              this.importBookInfo = res.data.data[0].book;
              this.importBookGroup = [];
              this.importBookChapters = res.data.data[0].chapters;
              this.showImportBookDialog = true;
            }
          }
        },
        error => {
          this.$message.error("上传书籍 " + (error && error.toString()));
        }
      );
      this.$refs.bookRef.value = null;
    },
    async importMultiBooks(books) {
      if (!books || !books.length) {
        return;
      }
      if (books.length == 1) {
        this.importBookInfo = books[0].book;
        this.importBookGroup = [];
        this.importBookChapters = books[0].chapters;
        this.showImportBookDialog = true;
        return;
      }
      const res = await this.$confirm(
        "你选择导入多本书籍，请选择导入方式?",
        "提示",
        {
          confirmButtonText: "批量导入",
          cancelButtonText: "逐一确认导入",
          type: "warning",
          closeOnClickModal: false,
          closeOnPressEscape: false,
          distinguishCancelAndClose: true
        }
      ).catch(action => {
        return action === "close" ? "close" : false;
      });
      if (res === "close") {
        return;
      }
      if (res) {
        const customImportBookInfo = await this.customImportBookInfo();
        if (customImportBookInfo === false) {
          return;
        }
        for (let i = 0; i < books.length; i++) {
          const book = books[i];
          await this.saveBook(
            { ...book.book, ...customImportBookInfo },
            true
          ).catch(() => {});
        }
      } else {
        for (let i = 0; i < books.length; i++) {
          const book = books[i];
          this.importMultiBookTip = `（${i + 1}/${books.length}）`;
          await this.waitForImportBook(book);
        }
        this.importMultiBookTip = "";
      }
    },
    waitForImportBook(bookInfo) {
      return new Promise(resolve => {
        this.importBookInfo = bookInfo.book;
        this.importBookGroup = [];
        this.importBookChapters = bookInfo.chapters;
        this.showImportBookDialog = true;
        this.$once("importEnd", resolve);
      });
    },
    importBookDialogClosed() {
      const url = this.importBookInfo.bookUrl;
      this.importBookInfo = {};
      this.importBookGroup = [];
      this.importBookChapters = [];
      this.importUsedTxtRule = "";
      this.$nextTick(() => {
        this.$emit("importEnd");
      });

      if (url && url.indexOf("assets") >= 0) {
        Axios.post(this.api + "/deleteFile", { url }, { silent: true }).then(
          () => {
            //
          },
          () => {
            //
          }
        );
      }
    },
    async customImportBookInfo(options, book) {
      const info = book || {};
      info.groupId = [];
      const items = [
        {
          name: "groupId",
          label: "分组",
          type: "select",
          placeholder: "请选择分组",
          options: this.bookGroupSetList.map(v => ({
            label: v.groupName,
            value: v.groupId
          }))
        }
      ];
      if (book && book.name) {
        items.unshift(
          { name: "name", label: "书名", type: "input" },
          { name: "author", label: "作者", type: "input" }
        );
      }
      const res = await this.$msgbox({
        title: "统一设置分组",
        message: this.renderForm(
          book ? "customImportBookInfo" : "customImportBookGroup",
          info,
          items,
          value => {
            if (book) {
              info.name = value.name;
              info.author = value.author;
            }
            info.groupId = value.groupId;
          }
        ),
        showCancelButton: true,
        confirmButtonText: "确定",
        cancelButtonText: "取消",
        ...(options || {})
      }).catch(action => {
        return action === "close" ? "close" : false;
      });
      if (res === "confirm") {
        return {
          group: info.groupId.reduce((group, value) => {
            return Long.fromNumber(group)
              .or(Long.fromNumber(value))
              .toNumber();
          }, 0)
        };
      }
      return false;
    },
    showBookManage() {
      eventBus.$emit("showBookManageDialog");
    },
    showManageBookGroup() {
      this.loadBookGroup(true);
      eventBus.$emit("showBookGroupDialog", false);
    },
    showActiveLicenseDialog() {
      eventBus.$emit("showActiveLicenseDialog");
    },
    showFileManagerDialog(home, title) {
      eventBus.$emit("showFileManagerDialog", home, title);
    },
    downloadBackupFile() {
      const url = buildURL(this.api + "/user/downloadBackupFile", {
        accessToken: this.$store.state.token
      });
      window.open(url, "__blank");
    },
    showBookmarkDialog() {
      eventBus.$emit("showBookmarkDialog");
    },
    showReplaceRuleDialog() {
      eventBus.$emit("showReplaceRuleDialog");
    },
    showLicense() {
      eventBus.$emit("showLicenseDialog");
    },
    getShowShelfBooks(bookGroup) {
      // 处理特殊分组
      if (bookGroup === -1) {
        // 全部
        return this.shelfBooks;
      } else if (bookGroup === -2) {
        // 本地
        return this.shelfBooks.filter(v => v.origin === "loc_book");
      } else if (bookGroup === -3) {
        // 音频
        return this.shelfBooks.filter(v => v.type === 1);
      } else if (bookGroup === -4) {
        // 未分组
        return this.shelfBooks.filter(v => v.group === 0);
      } else if (bookGroup === -5) {
        // 更新错误
        return this.shelfBooks.filter(v => v.canUpdate && v.lastCheckError);
      }

      return this.shelfBooks.filter(v => {
        if (bookGroup === 0) return true;
        return Long.fromNumber(v.group || 0)
          .and(Long.fromNumber(bookGroup))
          .greaterThan(0);
      });
    },
    loadRssSources(refresh) {
      return this.$root.$children[0].loadRssSources(refresh);
    },
    showRssDialog() {
      eventBus.$emit("showRssSourceListDialog");
    },
    showRssArticleListDialog(source) {
      eventBus.$emit("showRssArticleListDialog", source);
    },
    noop() {},
    exportBookSource() {
      Axios.get(this.api + "/getBookSources").then(
        res => {
          if (res.data.isSuccess) {
            const aEle = document.createElement("a");
            const blob = new Blob([
              JSON.stringify(res.data.data || [], null, 4)
            ]);

            aEle.download = "reader书源-" + this.currentDateTime() + ".json";
            aEle.href = URL.createObjectURL(blob);
            aEle.click();
          }
        },
        error => {
          this.$message.error("导出书源失败 " + (error && error.toString()));
        }
      );
    },
    async deleteAllBookSource() {
      const res = await this.$confirm("确认要清空所有书源吗?", "提示", {
        confirmButtonText: "确定",
        cancelButtonText: "取消",
        type: "warning"
      }).catch(() => {
        return false;
      });
      if (!res) {
        return;
      }
      Axios.post(this.api + "/deleteAllBookSources").then(
        res => {
          if (res.data.isSuccess) {
            //
            this.$message.success("清空书源成功");
            this.loadBookSource(true);
          }
        },
        error => {
          this.$message.error("清空书源失败 " + (error && error.toString()));
        }
      );
    },
    async deleteBookSourceFile() {
      const res = await this.$confirm("确认要恢复默认书源吗?", "提示", {
        confirmButtonText: "确定",
        cancelButtonText: "取消",
        type: "warning"
      }).catch(() => {
        return false;
      });
      if (!res) {
        return;
      }
      Axios.post(this.api + "/deleteBookSourcesFile").then(
        res => {
          if (res.data.isSuccess) {
            //
            this.$message.success("恢复默认书源成功");
            this.loadBookSource(true);
          }
        },
        error => {
          this.$message.error("操作失败 " + (error && error.toString()));
        }
      );
    },
    editBookSource(bookSource) {
      const editHandler = data => {
        eventBus.$emit(
          "showEditor",
          "编辑书源",
          JSON.stringify(data, null, 4),
          (content, close) => {
            try {
              const source = JSON.parse(content);
              if (!source.bookSourceName) {
                this.$message.error("书源名称不能为空");
                return;
              }
              if (!source.bookSourceUrl) {
                this.$message.error("书源链接不能为空");
                return;
              }
              Axios.post(this.api + "/saveBookSource", source).then(
                res => {
                  if (res.data.isSuccess) {
                    //
                    close();
                    this.$message.success("保存书源成功");
                    this.loadBookSource(true);
                  }
                },
                error => {
                  this.$message.error(
                    "保存书源失败 " + (error && error.toString())
                  );
                }
              );
            } catch (e) {
              this.$message.error("书源必须是JSON格式");
            }
          }
        );
      };
      if (!bookSource) {
        editHandler({
          bookSourceComment: "",
          bookSourceGroup: "",
          bookSourceName: "新增书源",
          bookSourceType: 0,
          bookSourceUrl: "",
          bookUrlPattern: "",
          enabled: true,
          header: "",
          enabledExplore: true,
          exploreUrl: "",
          ruleBookInfo: {},
          ruleContent: {
            content: ""
          },
          ruleExplore: {},
          ruleSearch: {
            author: "",
            bookList: "",
            bookUrl: "",
            coverUrl: "",
            intro: "",
            kind: "",
            lastChapter: "",
            name: ""
          },
          ruleToc: {
            chapterList: "",
            chapterName: "",
            chapterUrl: ""
          },
          searchUrl: ""
        });
        return;
      }
      Axios.post(this.api + "/getBookSource", {
        bookSourceUrl: bookSource.bookSourceUrl
      }).then(
        res => {
          if (res.data.isSuccess) {
            //
            editHandler(res.data.data);
          }
        },
        error => {
          this.$message.error(
            "加载书源信息失败 " + (error && error.toString())
          );
        }
      );
    },
    editBookSourceHeader(bookSource) {
      const editHandler = book => {
        eventBus.$emit(
          "showEditor",
          "编辑书源请求头header",
          book.header || JSON.stringify({ Cookie: "" }, null, 4),
          (content, close) => {
            try {
              if (!content.startsWith("@js:") && !content.startsWith("<js>")) {
                JSON.parse(content);
              }
              book.header = content;
              Axios.post(this.api + "/saveBookSource", book).then(
                res => {
                  if (res.data.isSuccess) {
                    close();
                    this.$message.success("保存书源成功");
                    this.loadBookSource(true);
                  }
                },
                error => {
                  this.$message.error(
                    "保存书源失败 " + (error && error.toString())
                  );
                }
              );
            } catch (e) {
              this.$message.error("书源请求头必须是JSON格式/脚本格式");
            }
          }
        );
      };
      Axios.post(this.api + "/getBookSource", {
        bookSourceUrl: bookSource.bookSourceUrl
      }).then(
        res => {
          if (res.data.isSuccess) {
            editHandler(res.data.data);
          }
        },
        error => {
          this.$message.error(
            "加载书源信息失败 " + (error && error.toString())
          );
        }
      );
    },
    updateForce() {
      if ("serviceWorker" in navigator) {
        navigator.serviceWorker
          .getRegistrations()
          .then(async function(registrations) {
            /* eslint-disable-next-line no-console */
            console.log("registrations", registrations);
            for (let i = 0; i < registrations.length; i++) {
              await registrations[i].update();
            }

            /* eslint-disable-next-line no-console */
            console.log("Try to clear home cache");
            navigator.serviceWorker.controller &&
              navigator.serviceWorker.controller.postMessage({
                type: "CLEAR_HOME_CACHE"
              });

            /* eslint-disable-next-line no-console */
            console.log("Try to skip waiting");
            navigator.serviceWorker.controller &&
              navigator.serviceWorker.controller.postMessage({
                type: "SKIP_WAITING"
              });

            setTimeout(() => {
              /* eslint-disable-next-line no-console */
              console.log("Try to reload force");
              window.location.reload(true);
            }, 50);
          });
      }
    },
    async scanCacheStorage() {
      this.localCacheStats = {
        total: (await this.analyseLocalStorage()).totalBytes,
        bookSourceList: (await this.analyseLocalStorage("bookSourceList"))
          .totalBytes,
        rssSources: (await this.analyseLocalStorage("rssSources")).totalBytes,
        chapterList: (await this.analyseLocalStorage("chapterList")).totalBytes,
        chapterContent: (await this.analyseLocalStorage("chapterContent"))
          .totalBytes,
        ttsData: (await this.analyseLocalStorage("ttsData")).totalBytes
      };
    },
    analyseLocalStorage(match) {
      let totalBytes = 0;
      let cacheBytes = 0;
      return window.$cacheStorage
        .iterate(function(value, key) {
          if (!match || key.indexOf(match) >= 0) {
            totalBytes += JSON.stringify(value).getBytesLength();
            if (key.startsWith("localCache@")) {
              cacheBytes += JSON.stringify(value).getBytesLength();
            }
          }
        })
        .then(() => {
          return {
            totalBytes: formatSize(totalBytes),
            cacheBytes: formatSize(cacheBytes)
          };
        })
        .catch(function() {
          // 当出错时，此处代码运行
          // console.log(err);
        });
    },
    clearCache(match) {
      let cacheBytes = 0;
      window.$cacheStorage
        .iterate(function(value, key) {
          if (!match || key.indexOf(match) >= 0) {
            if (key.startsWith("localCache@")) {
              cacheBytes += JSON.stringify(value).getBytesLength();
              window.$cacheStorage.removeItem(key);
            }
          }
        })
        .then(() => {
          this.scanCacheStorage();

          return {
            cacheBytes: formatSize(cacheBytes)
          };
        })
        .catch(function() {
          // 当出错时，此处代码运行
          // console.log(err);
        });
    },
    scrollHandler() {
      this.lastScrollTop = this.$refs.bookList.scrollTop;
    },
    getBookCoverUrl(book) {
      return book.customCoverUrl || book.coverUrl;
    },
    logout() {
      Axios.post(this.api + "/logout").then(
        res => {
          if (res.data.isSuccess) {
            this.$store.commit("setToken", "");
            window.location.reload(true);
          }
        },
        error => {
          this.$message.error("注销失败 " + (error && error.toString()));
        }
      );
    },
    getChapterListByRule() {
      return Axios.post("/getChapterListByRule", this.importBookInfo).then(
        res => {
          if (res.data.isSuccess && res.data.data.book) {
            this.importBookInfo = res.data.data.book;
            this.importBookChapters = res.data.data.chapters;
          }
        },
        error => {
          this.$message.error("注销失败 " + (error && error.toString()));
        }
      );
    },
    showUserManageDialog() {
      eventBus.$emit("showUserManageDialog");
    },
    showMPCode() {
      eventBus.$emit("showMPCodeDialog");
    },
    joinTGChannel() {
      window.open("https://t.me/facker_channel", "_target");
    },
    ensureLoadBookCover() {
      // 手动触发滚动事件，显示书籍封面图片
      this.$refs.bookList.dispatchEvent(new MouseEvent("scroll"));

      // 上面一步应该能搞定，下面再确认一下
      this.$refs.bookListComp.$refs.bookCoverList.forEach(v => {
        if (!v.show && isInContainer(v.$el, this.$refs.bookList)) {
          v.show = true;
        }
      });
    }
  },
  computed: {
    ...mapGetters([
      "collapseMenu",
      "dialogWidth",
      "dialogSmallWidth",
      "dialogTop",
      "dialogContentHeight",
      "popupWidth"
    ]),
    config() {
      return this.$store.getters.config;
    },
    isNight() {
      return this.$store.getters.isNight;
    },
    themeColor() {
      if (this.$store.getters.isNight) {
        return {
          background: "#f7f7f7"
        };
      } else {
        return {
          background: "#222"
        };
      }
    },
    bookList() {
      if (this.isSearchResult) return this.searchResult;
      const q = this.keyword;
      if (!q) return this.showShelfBooks;
      return this.showShelfBooks.filter(
        v =>
          (v.name || "").indexOf(q) >= 0 ||
          (v.author || "").indexOf(q) >= 0 ||
          (v.kind || "").indexOf(q) >= 0
      );
    },
    bookCoverList() {
      return this.bookList
        .filter(v => this.getBookCoverUrl(v))
        .map(v => this.getCover(this.getBookCoverUrl(v), true));
    },
    shelfBooks() {
      const books = [].concat(this.$store.getters.shelfBooks);
      return books.sort((a, b) => {
        const compare =
          a[this.shelfConfig.bookOrder] > b[this.shelfConfig.bookOrder]
            ? 1
            : a[this.shelfConfig.bookOrder] < b[this.shelfConfig.bookOrder]
            ? -1
            : 0;
        return compare * (this.shelfConfig.bookOrder === "name" ? 1 : -1);
      });
    },
    showShelfBooks() {
      return this.getShowShelfBooks(this.showBookGroup);
    },
    searchResultMap() {
      return this.searchResult.reduce((c, v) => {
        c[v.bookUrl] = v;
        return c;
      }, {});
    },
    connectStatus() {
      return this.$store.state.connected
        ? `后端已连接`
        : this.connecting
        ? "正在连接后端服务器……"
        : "点击设置后端接口前缀";
    },
    connectType() {
      return this.$store.state.connected ? "success" : "danger";
    },
    readingRecent() {
      return this.$store.getters.readingBook &&
        this.$store.getters.readingBook.name
        ? this.$store.getters.readingBook
        : {
            name: "尚无阅读记录",
            bookUrl: "",
            index: 0
          };
    },
    loginAuth() {
      return this.$store.state.loginAuth;
    },
    bookSourceList() {
      return this.$store.state.bookSourceList;
    },
    userNS: {
      get() {
        return this.$store.state.userNS;
      },
      set(val) {
        this.$store.commit("setUserNS", val);
        if (val) {
          this.$store.commit("setIsManagerMode", true);
        }
      }
    },
    userList: {
      get() {
        return this.$store.state.userList;
      },
      set(val) {
        this.$store.commit("setUserList", val);
      }
    },
    bookSourceShowList() {
      return this.isShowFailureBookSource
        ? this.$store.state.failureBookSource
        : this.bookSourceList;
    },
    bookSourceGroupList() {
      return this.$store.getters.bookSourceGroupList;
    },
    bookSourceShowGroup() {
      if (this.isShowFailureBookSource) {
        return ["全部"].concat(errorTypeList).concat(["timeout"]);
      }
      const groups = new Set(["全部"]);
      this.bookSourceShowList.forEach(v => {
        if (v.bookSourceGroup) {
          v.bookSourceGroup.split(",").forEach(group => {
            if (group) groups.add(group);
          });
        }
      });
      groups.add("未分组");
      return Array.from(groups);
    },
    bookSourceShowSearchResult() {
      if (!this.bookSourceKeyword) return this.bookSourceShowList;
      return this.bookSourceShowList.filter(
        v =>
          (v.bookSourceName || "").indexOf(this.bookSourceKeyword) >= 0 ||
          (v.bookSourceUrl || "").indexOf(this.bookSourceKeyword) >= 0
      );
    },
    bookSourceShowLength() {
      return this.bookSourceShowResult.length;
    },
    bookSourceShowResult() {
      if (!this.showSourceGroup || this.showSourceGroup === "全部") {
        return this.bookSourceShowSearchResult;
      }
      if (this.isShowFailureBookSource) {
        return this.bookSourceShowSearchResult.filter(
          v => !this.showSourceGroup || v.errorMsg.indexOf(this.showSourceGroup) >= 0
        );
      }
      return this.bookSourceShowSearchResult.filter(v =>
        this.showSourceGroup === "未分组"
          ? !v.bookSourceGroup
          : (v.bookSourceGroup + ",").indexOf(this.showSourceGroup + ",") >= 0
      );
    },
    bookSourceShowResultPageList() {
      const start =
        (this.bookSourcePagination.page - 1) * this.bookSourcePagination.size;
      if (start > this.bookSourceShowResult.length) {
        return [];
      }
      return this.bookSourceShowResult.slice(
        start,
        Math.min(
          start + this.bookSourcePagination.size,
          this.bookSourceShowResult.length
        )
      );
    },
    shelfConfig() {
      return this.$store.state.shelfConfig;
    },
    showBookGroup: {
      get() {
        if (!this.bookGroupDisplayList.length) return -1;
        return this.$store.state.shelfConfig.showBookGroup;
      },
      set(val) {
        this.$store.commit("setShelfConfig", {
          ...this.$store.state.shelfConfig,
          showBookGroup: val
        });
      }
    },
    showBookGroupString: {
      get() {
        return "" + this.showBookGroup;
      },
      set(val) {
        this.showBookGroup = +val;
      }
    },
    bookGroupSetList() {
      return this.$store.state.bookGroupList.filter(v => v.groupId > 0);
    },
    bookGroupDisplayList() {
      return this.$store.state.bookGroupList
        .filter(v => this.getShowShelfBooks(v.groupId).length && v.show)
        .sort((a, b) => a.order - b.order);
    },
    searchConfig: {
      get() {
        return this.$store.state.searchConfig;
      },
      set(val) {
        this.$store.commit("setSearchConfig", val);
      }
    },
    isShowTocRule() {
      try {
        return (
          this.importBookInfo &&
          this.importBookInfo.originName &&
          (this.importBookInfo.originName.toLowerCase().endsWith(".txt") ||
            this.importBookInfo.originName.toLowerCase().endsWith(".epub") ||
            this.importBookInfo.originName.toLowerCase().endsWith(".pdf"))
        );
      } catch (e) {
        // console.log(e);
      }
      return false;
    },
    tocRuleList() {
      if (!this.importBookInfo || !this.importBookInfo.originName) {
        return [];
      }
      if (this.importBookInfo.originName.toLowerCase().endsWith(".txt")) {
        // txt
        return this.$store.state.txtTocRules;
      } else if (
        this.importBookInfo.originName.toLowerCase().endsWith(".epub")
      ) {
        // epub
        return [
          { name: "根据 Spin 获取章节，使用 Toc 补充章节名", rule: "spin+toc" },
          { name: "根据 Spin 获取章节，强制使用 Toc 章节名", rule: "spin<toc" },
          { name: "根据 Spin 获取章节", rule: "spin" },
          { name: "根据 Toc 获取章节，使用 Spin 补充章节名", rule: "toc+spin" },
          { name: "根据 Toc 获取章节，强制使用 Spin 章节名", rule: "toc<spin" },
          { name: "根据 Toc 获取章节", rule: "toc" }
        ];
      } else {
        return [
          { name: "使用书签作为章节", rule: "outline" },
          { name: "一页一章", rule: "page" }
        ];
      }
    },
    isShowActiveLicenseBtn() {
      return (
        this.$store.state.isManagerMode &&
        window.location.host.indexOf("htmake") >= 0
      );
    }
  }
};
</script>

<style lang="stylus" scoped>
.index-wrapper {
  height: 100%;
  width: 100%;
  display: flex;
  flex-direction: row;

  .navigation-wrapper {
    width: 260px;
    min-width: 260px;
    height: 100%;
    box-sizing: border-box;
    background-color: #F7F7F7;
    position: relative;
    padding-top: 0;
    padding-top: constant(safe-area-inset-top) !important;
    padding-top: env(safe-area-inset-top) !important;

    .navigation-inner-wrapper {
      padding: 48px 36px 66px 36px;
      height: 100%;
      overflow-y: auto;
      box-sizing: border-box;
    }

    .navigation-title {
      font-size: 24px;
      font-weight: 600;
      font-family: -apple-system, "Noto Sans", "Helvetica Neue", Helvetica, "Nimbus Sans L", Arial, "Liberation Sans", "PingFang SC", "Hiragino Sans GB", "Noto Sans CJK SC", "Source Han Sans SC", "Source Han Sans CN", "Microsoft YaHei", "Wenquanyi Micro Hei", "WenQuanYi Zen Hei", "ST Heiti", SimHei, "WenQuanYi Zen Hei Sharp", sans-serif;

      .version-text {
        float: right;
        font-size: 14px;
        line-height: 33px;
        font-weight: 400;
        color: #b1b1b1;
        display: inline-block;
        cursor: pointer;
      }
    }

    .navigation-sub-title {
      font-size: 16px;
      font-weight: 500;
      font-family: -apple-system, "Noto Sans", "Helvetica Neue", Helvetica, "Nimbus Sans L", Arial, "Liberation Sans", "PingFang SC", "Hiragino Sans GB", "Noto Sans CJK SC", "Source Han Sans SC", "Source Han Sans CN", "Microsoft YaHei", "Wenquanyi Micro Hei", "WenQuanYi Zen Hei", "ST Heiti", SimHei, "WenQuanYi Zen Hei Sharp", sans-serif;
      margin-top: 16px;
      color: #b1b1b1;
    }

    .search-wrapper {
      .search-input {
        border-radius: 50%;
        margin-top: 24px;

        >>> .el-input__inner {
          border-radius: 50px;
          border-color: #E3E3E3;
        }
      }
    }

    .recent-wrapper {
      margin-top: 36px;

      .recent-title {
        font-size: 14px;
        color: #b1b1b1;
        font-family: -apple-system, "Noto Sans", "Helvetica Neue", Helvetica, "Nimbus Sans L", Arial, "Liberation Sans", "PingFang SC", "Hiragino Sans GB", "Noto Sans CJK SC", "Source Han Sans SC", "Source Han Sans CN", "Microsoft YaHei", "Wenquanyi Micro Hei", "WenQuanYi Zen Hei", "ST Heiti", SimHei, "WenQuanYi Zen Hei Sharp", sans-serif;
      }

      .reading-recent {
        margin: 18px 0;

        .recent-book {
          cursor: pointer;
          max-width: 100%;
          overflow: hidden;
          text-overflow: ellipsis;
        }
      }
    }

    .setting-wrapper {
      margin-top: 36px;

      .setting-title {
        font-size: 14px;
        color: #b1b1b1;
        font-family: -apple-system, "Noto Sans", "Helvetica Neue", Helvetica, "Nimbus Sans L", Arial, "Liberation Sans", "PingFang SC", "Hiragino Sans GB", "Noto Sans CJK SC", "Source Han Sans SC", "Source Han Sans CN", "Microsoft YaHei", "Wenquanyi Micro Hei", "WenQuanYi Zen Hei", "ST Heiti", SimHei, "WenQuanYi Zen Hei Sharp", sans-serif;

        .right-text {
          float: right;
          display: inline-block;
          height: 20px;
          line-height: 20px;
          cursor: pointer;
          user-select: none;
        }
      }

      .no-point {
        pointer-events: none;
      }

      .setting-connect {
        cursor: pointer;
        max-width: 100%;
        overflow: hidden;
        text-overflow: ellipsis;
      }

      .setting-item {
        padding-top: 16px;
      }

      .setting-btn {
        margin-right: 15px;
        margin-bottom: 15px;
        cursor: pointer;
      }

      .setting-select {
        width: 100%;
      }
    }

    .search-setting {
      margin-top: 28px;
    }

    .bottom-icons {
      position: absolute;
      bottom: 30px;
      width: 188px;
      left: 36px;
      align-items: center;
      display: flex;
      flex-direction: row;
      justify-content: space-between;
      pointer-events: none;

      .bottom-icon {
        height: 36px;
        pointer-events: all;
        img {
          width: 36px;
          height: 36px;
        }
      }

      .theme-item {
        line-height: 32px;
        width: 36px;
        height: 36px;
        border-radius: 100%;
        display: inline-block;
        cursor: pointer;
        text-align: center;
        vertical-align: middle;
        pointer-events: all;

        .el-icon-moon {
          color: #f7f7f7;
          line-height: 34px;
        }
        .el-icon-sunny {
          color: #121212;
          line-height: 34px;
        }
      }
    }

    .setting-wrapper:nth-last-child(1) {
      padding-bottom: 20px;
    }
  }

  .shelf-wrapper {
    padding: 48px 48px;
    height: 100%;
    max-height: 100%;
    width: 100%;
    background-color: #fff;
    display: flex;
    flex-direction: column;
    box-sizing: border-box;

    .shelf-title {
      font-size: 20px;
      font-weight: 600;
      font-family: -apple-system, "Noto Sans", "Helvetica Neue", Helvetica, "Nimbus Sans L", Arial, "Liberation Sans", "PingFang SC", "Hiragino Sans GB", "Noto Sans CJK SC", "Source Han Sans SC", "Source Han Sans CN", "Microsoft YaHei", "Wenquanyi Micro Hei", "WenQuanYi Zen Hei", "ST Heiti", SimHei, "WenQuanYi Zen Hei Sharp", sans-serif;
      margin-bottom: 5px;
      min-width: 320px;
      box-sizing: border-box;

      .el-icon-menu {
        cursor: pointer;
      }

      .title-btn {
        font-size: 14px;
        line-height: 28px;
        float: right;
        cursor: pointer;
        user-select: none;
        margin-left: 10px;

        >>>.el-icon-loading {
          font-size: 16px;
        }
      }
    }

    >>>.el-icon-loading {
      font-size: 36px;
      color: #B5B5B5;
    }

    >>>.el-loading-text {
      font-weight: 500;
      color: #B5B5B5;
    }

    .book-group-wrapper {
      padding: 5px 0;
      margin-bottom: 10px;

      .book-group-tabs {
        width: 100%;
      }

      .book-group-btn {
        margin-right: 10px;
        cursor: pointer;
      }

      .book-group-btn.selected {
        color: #fff;
        background: #409EFF;
        border-color: #409EFF;
      }
    }

    .books-wrapper {
      flex: 1;
      overflow-x: hidden;
      overflow-y: scroll;
    }

    .books-wrapper::-webkit-scrollbar {
      width: 0 !important;
    }
  }
}

.unread-num-badge {
  >>>.el-badge__content {
    border: none;
  }
}

.night {
  >>>.navigation-wrapper {
    background-color: #121212;
    border-right: 1px solid #555;
  }
  >>>.navigation-title {
    color: #bbb;
  }
  >>>.shelf-title {
    color: #bbb;
  }
  >>>.shelf-wrapper {
    background-color: #222;
  }
  >>>.el-input__inner {
    background-color: #444;
    border: 1px solid #444 !important;
    color: #aaa;
  }

  >>>.check-tip {
    color: #bbb;
  }
}

.source-container {
  padding: 0 10px;

  &.table-container {
    padding: 0;
  }

  .check-form {
    display: flex;
    flex-direction: row;
    overflow-x: auto;
    align-items: center;

    .check-form-label {
      min-width: 60px;
    }

    .el-input {
      width: auto;
      min-width: 100px;
      margin-right: 10px;
    }

    .el-input-number {
      min-width: 130px;
      margin-right: 10px;
    }

    .book-cover {
      width: 84px;
      height: 112px;

      .cover {
        width: 84px;
        height: 112px;
      }
    }

    .book-info {
      display: flex;
      flex-direction: column;
      margin-left: 30px;
      justify-content: space-between;
      min-height: 100px;

      .toc-refresh-btn {
        margin-left: 5px;
      }

      span {
        display: inline-block;
        min-width: 56px;
        text-align-last: justify;
      }
      .el-input {
        width: auto;
        min-width: 100px;
        margin-right: 10px;
      }
      .el-input-number {
        min-width: 130px;
        margin-right: 10px;
      }
    }
  }

  .chapter-title {
    font-size: 15px;
    padding: 5px 0;
    font-weight: 600;
    margin-top: 10px;
  }

  .chapter-list {
    overflow-y: auto;
    box-sizing: border-box;
    padding: 0 5px;

    p {
      margin-top: 0.4em;
      margin-bottom: 0.4em;
    }
  }

  .source-group-wrapper {
    display: flex;
    flex-direction: row;
    overflow-x: auto;
    padding: 5px 0;

    .booksource-group-tabs {
      width: 100%;
    }

    .source-group-btn {
      margin-right: 10px;
      cursor: pointer;
    }

    .source-group-btn.selected {
      color: #fff;
      background: #409EFF;
      border-color: #409EFF;
    }
  }

  .el-pagination {
    margin-top: 8px;
    float: right;
    max-width: 100%;
    overflow-x: auto;
    box-sizing: border-box;
  }

  >>>.source-checkbox {
    display: block;
    padding: 8px 0;
    width: 100%;
  }

  pre {
    margin: 0;
  }

  .source-pagination::after {
    display: table;
    content: "";
    clear: both;
  }
}

.source-list-container {
  max-height: calc(var(--vh, 1vh) * 70 - 54px - 60px - 66px);
  overflow-y: auto;
  overflow-x: auto;
}

.night {
  .source-container {
    .source-group-wrapper {
      .source-group-btn.selected {
        color: #fff;
        background: #185798;
        border-color: #185798;
      }
    }
  }
  .book-group-wrapper {
    .book-group-btn.selected {
      color: #fff;
      background: #185798 !important;
      border-color: #185798 !important;
    }
  }
}

.source-container::-webkit-scrollbar {
  width: 0 !important;
}
.navigation-inner-wrapper::-webkit-scrollbar {
  width: 0 !important;
}
>>> .el-table__body-wrapper::-webkit-scrollbar {
  width: 0 !important;
}
>>> .el-dialog__wrapper::-webkit-scrollbar {
  width: 0 !important;
}

@media screen and (max-width: 750px) {
  .index-wrapper {
    overflow-x: hidden;

    >>>.navigation-wrapper {
      .navigation-inner-wrapper {
        padding: 20px 36px 66px 36px;
      }
    }
    >>>.shelf-wrapper {
      padding: 0;
      padding-top: constant(safe-area-inset-top) !important;
      padding-top: env(safe-area-inset-top) !important;

      .shelf-title {
        padding: 20px 24px 0 24px;
      }

      .book-group-wrapper {
        margin-left: 24px;
        margin-right: 24px;
      }
    }
  }
  .source-list-container  {
    max-height: calc(var(--vh, 1vh) * 100 - 54px - 40px - 66px);
  }
}
@media screen and (max-width: 480px) {
  .source-container.table-container {
    margin: -15px -5px;
  }
}
</style>
<style>
.navigation-hidden {
  margin-left: -260px;
}
.navigation-in {
  margin-left: 0px;
  transition: margin-left 0.3s;
}
.navigation-out {
  margin-left: -260px;
  transition: margin-left 0.3s;
}
.popper-intro {
  padding: 15px;
}
.book-kind span {
  display: inline-block;
  margin-left: 5px;
  margin-right: 5px;
}
.night-theme .popper-intro {
  background: #121212;
  color: #bbb !important;
  border: none;
}
.night-theme .popper-intro.el-popper[x-placement^="bottom"] .popper__arrow,
.night-theme
  .popper-intro.el-popper[x-placement^="bottom"]
  .popper__arrow::after {
  border-bottom-color: #121212 !important;
}
.night-theme .popper-intro.el-popper[x-placement^="top"] .popper__arrow,
.night-theme .popper-intro.el-popper[x-placement^="top"] .popper__arrow::after {
  border-top-color: #121212 !important;
}
.night-theme .el-popover__title {
  color: #ddd !important;
}
.status-bar-light-bg {
  background-image: linear-gradient(
    to bottom,
    rgba(0, 0, 0, 0.2) 0,
    transparent 36px
  ) !important;
}
.status-bar-light-bg-dialog .el-dialog.is-fullscreen {
  background-image: linear-gradient(
    to bottom,
    rgba(0, 0, 0, 0.2) 0,
    transparent 36px
  ) !important;
}
@media (hover: hover) {
  .book:hover {
    background: rgba(0, 0, 0, 0.1);
    transition-duration: 0.5s;
  }
  .el-icon-close:hover {
    color: #409eff;
  }
  .el-icon-edit:hover {
    color: #409eff;
  }
}

.mini-interface .el-dialog__body {
  padding: 15px 20px;
}
.book-group-tabs .el-tabs__header {
  margin-bottom: 0px;
}

/* 虚拟列表下拉选择（单源搜索书源选择） */
.el-popover.el-popper.select-virtual-list-popover {
  height: 300px;
  padding: 0;
  border: 1px solid #e4e7ed;
  border-radius: 4px;
  background-color: #fff;
  box-shadow: 0 2px 12px 0 rgba(0, 0, 0, 0.1);
  box-sizing: border-box;
}
.el-popover.el-popper.select-virtual-list-popover .virtual-list {
  width: 100%;
  height: calc(100% - 20px);
  padding: 10px 0;
  overflow-y: auto;
}
.virtual-item {
  font-size: 14px;
  padding: 0 20px;
  position: relative;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: #606266;
  height: 32px;
  line-height: 32px;
  box-sizing: border-box;
  cursor: pointer;
}
.virtual-item:hover {
  background-color: #eee;
}
.virtual-item.is-selected {
  color: #409eff;
  background-color: #dbeeff;
}
.virt-list__client {
  width: 100%;
  height: 100%;
  overflow: auto;
}

/* 书架书籍列表（列表视图 / 虚拟列表视图） */
.wrapper {
  display: grid;
  grid-template-columns: repeat(auto-fill, 380px);
  justify-content: space-around;
  grid-gap: 10px;
  height: 100%;
}
.wrapper .book {
  user-select: none;
  display: flex;
  cursor: pointer;
  margin-bottom: 18px;
  padding: 24px 24px;
  width: 360px;
  flex-direction: row;
  justify-content: space-around;
}
.wrapper .book .cover-img,
.wrapper .book .cover-img .cover {
  width: 84px;
  height: 112px;
}
.wrapper .book .info {
  position: relative;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  align-items: left;
  height: 112px;
  margin-left: 20px;
  flex: 1;
}
.wrapper .book .info .book-operation {
  position: absolute;
  right: 5px;
  top: 0px;
  font-size: 24px;
  color: #969ba3;
}
.wrapper .book .info .book-operation i {
  margin-left: 10px;
}
.wrapper .book .info .name {
  width: fit-content;
  font-size: 16px;
  font-weight: 700;
  color: #33373d;
  margin-right: 38px;
  max-height: 45px;
  word-wrap: break-word;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
}
.wrapper .book .info .name.edit {
  margin-right: 62px;
}
.wrapper .book .info .sub {
  display: flex;
  flex-direction: row;
  font-size: 12px;
  font-weight: 600;
  color: #969ba3;
}
.wrapper .book .info .sub .dot {
  margin: 0 7px;
}
.wrapper .book .info .intro,
.wrapper .book .info .dur-chapter,
.wrapper .book .info .last-chapter {
  color: #6b6b6b;
  font-size: 13px;
  margin-top: 3px;
  font-weight: 500;
  word-wrap: break-word;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 1;
  text-align: left;
}
.wrapper .book .info .source-count {
  color: #6b6b6b;
  font-size: 14px;
  display: inline-block;
  margin-left: 10px;
  cursor: pointer;
}
.wrapper:last-child {
  margin-right: auto;
}
.night .book .info .name {
  color: #bbb !important;
}
.night .book .info .book-operation,
.night .book .info .sub {
  color: #6b6b6b !important;
}
.night .book .info .intro,
.night .book .info .dur-chapter,
.night .book .info .last-chapter {
  color: #969ba3 !important;
}
@media (hover: hover) {
  .wrapper .book:hover {
    background: rgba(0, 0, 0, 0.1);
    transition-duration: 0.5s;
  }
}
@media screen and (max-width: 750px) {
  .wrapper {
    display: flex;
    flex-direction: column;
  }
  .wrapper .book {
    box-sizing: border-box;
    width: 100%;
    margin-bottom: 0;
    padding: 10px 20px;
  }
}

/* 书架书籍列表（网格视图） */
.wrapper[style*="grid-template-columns"] {
  display: grid;
  justify-content: space-around;
  grid-gap: 15px;
  padding: 0 20px;
}
.wrapper .book[style*="--book-item-width"] {
  display: flex;
  cursor: pointer;
  padding: 0;
  width: var(--book-item-width, 100px);
  flex-direction: column;
  box-sizing: border-box;
  position: relative;
}
.wrapper .book[style*="--book-item-width"] .cover-img,
.wrapper .book[style*="--book-item-width"] .cover-img .cover {
  width: 100%;
  height: calc(var(--book-item-width, 100px) * 1.5);
}
.wrapper .book[style*="--book-item-width"] .name {
  width: 100%;
  text-align: center;
  font-size: 14px;
  font-weight: 700;
  color: #33373d;
  max-height: 40px;
  word-wrap: break-word;
  overflow: hidden;
  margin-top: 5px;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
}
.wrapper .book[style*="--book-item-width"] .book-operation {
  position: absolute;
  right: 0;
  top: 0;
  font-size: 24px;
  color: #969ba3;
}
.wrapper .book[style*="--book-item-width"] .book-operation i {
  margin-left: 10px;
  display: block;
  color: #8abcff;
}
.night .wrapper .book[style*="--book-item-width"] .name {
  color: #bbb !important;
}

/* 探索书源（书海） */
.popup-wrapper {
  margin: -16px;
  margin-bottom: -13px;
  padding: 24px;
  padding-top: calc(24px + constant(safe-area-inset-top));
  padding-top: calc(24px + env(safe-area-inset-top));
}
.popup-wrapper .title-zone {
  margin: 0 0 20px 0;
  width: 100%;
  display: flex;
  flex-direction: row;
  flex-wrap: wrap;
  justify-content: space-between;
}
.popup-wrapper .title {
  font-size: 18px;
  font-weight: 400;
  font-family: -apple-system, "Noto Sans", "Helvetica Neue", Helvetica, "Nimbus Sans L", Arial, "Liberation Sans", "PingFang SC", "Hiragino Sans GB", "Noto Sans CJK SC", "Source Han Sans SC", "Source Han Sans CN", "Microsoft YaHei", "Wenquanyi Micro Hei", "WenQuanYi Zen Hei", "ST Heiti", SimHei, "WenQuanYi Zen Hei Sharp", sans-serif;
  color: #ed4259;
  border-bottom: 1px solid #ed4259;
  width: fit-content;
}
.popup-wrapper .title-btn {
  font-size: 14px;
  line-height: 26px;
  color: #ed4259;
}
.popup-wrapper .title-btn .source-count {
  display: inline-block;
  margin-right: 25px;
  color: #606266;
}
.popup-wrapper .title-btn .close-btn {
  font-size: 20px;
  vertical-align: middle;
  cursor: pointer;
}
.popup-wrapper .title-btn.loading {
  color: #606266;
}
.popup-wrapper .source-group-wrapper {
  display: flex;
  flex-direction: row;
  overflow-x: auto;
  padding: 5px 0;
}
.popup-wrapper .source-group-wrapper .booksource-group-tabs {
  width: 100%;
}
.popup-wrapper .source-group-wrapper .source-group-btn {
  margin-right: 10px;
  cursor: pointer;
}
.popup-wrapper .source-group-wrapper .source-group-btn.selected {
  color: #ed4259;
}
.popup-wrapper .data-wrapper {
  height: 300px;
  overflow: auto;
}
.popup-wrapper .data-wrapper .cata {
  display: flex;
  flex-direction: row;
  flex-wrap: wrap;
  justify-content: space-between;
}
.popup-wrapper .data-wrapper .cata .source-collapse {
  width: 100%;
  border: none;
}
.popup-wrapper .data-wrapper .cata .explore-group {
  display: flex;
  justify-content: space-between;
  margin-bottom: 2px;
  padding-top: 2px;
  overflow-x: auto;
}
.popup-wrapper .data-wrapper .cata .explore-btn {
  margin-right: 15px;
  margin-bottom: 5px;
  cursor: pointer;
}
.popup-wrapper .data-wrapper::-webkit-scrollbar {
  width: 0 !important;
}
.popup-wrapper .el-collapse-item__header,
.popup-wrapper .el-collapse-item__wrap {
  background: rgba(0, 0, 0, 0);
  color: #606266;
}
.popup-wrapper .el-collapse-item__content {
  color: #606266;
  padding: 10px;
}
.popup-wrapper .night .el-collapse-item__header,
.popup-wrapper .night .el-collapse-item__wrap {
  border-bottom: 1px solid #666;
}
.popup-wrapper .night .explore-group {
  border-bottom: 1px dashed #333;
}
.popup-wrapper .day .el-collapse-item__header,
.popup-wrapper .day .el-collapse-item__wrap {
  border-bottom: 1px solid #ebeef5;
}
.popup-wrapper .day .explore-group {
  border-bottom: 1px dashed #efefef;
}
</style>
